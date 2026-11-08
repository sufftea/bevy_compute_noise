use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::{
    prelude::*,
    render::{
        render_resource::{
            binding_types::{uniform_buffer, uniform_buffer_sized},
            BindGroup, BindGroupLayout, BindGroupLayoutEntries, Buffer, CachedComputePipelineId,
            DynamicBindGroupLayoutEntries, ShaderStages, TextureDimension,
        },
        renderer::RenderDevice,
    },
};

use crate::{
    image::ComputeNoiseSize,
    noise::{ComputeNoise, ErasedComputeNoise},
    render::pipeline::{noise_texture_2d, ComputeNoisePipelineKey, NoiseOp},
    shader::generate_noise_shader,
};

pub(crate) struct ComputeNoiseInstruction {
    pub images: Vec<Handle<Image>>,
    pub noise: ErasedComputeNoise,
    pub op: NoiseOp,
}
pub struct ComputeNoiseSequence {
    pub(crate) output: Handle<Image>,
    pub(crate) instructions: Vec<ComputeNoiseInstruction>,
}

impl ComputeNoiseSequence {
    pub fn key_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        for instruction in &self.instructions {
            instruction.noise.type_id.hash(&mut hasher);
        }
        hasher.finish()
    }
}

pub enum QueueNoiseOp {
    Generate(ErasedComputeNoise),
    Modify(Option<Handle<Image>>, ErasedComputeNoise),
    Combine(
        Option<Handle<Image>>,
        Option<Handle<Image>>,
        ErasedComputeNoise,
    ),
}

impl<T: ComputeNoise> From<T> for QueueNoiseOp {
    fn from(noise: T) -> Self {
        let erased = ErasedComputeNoise::from(noise);
        match T::NOISE_OP {
            NoiseOp::Generator => QueueNoiseOp::Generate(erased),
            NoiseOp::Modifier => QueueNoiseOp::Modify(None, erased),
            NoiseOp::Combiner => QueueNoiseOp::Combine(None, None, erased),
        }
    }
}

pub trait IntoNoiseSequence {
    fn into_sequence(self, output: Handle<Image>) -> ComputeNoiseSequence;
}

impl IntoNoiseSequence for QueueNoiseOp {
    fn into_sequence(self, output: Handle<Image>) -> ComputeNoiseSequence {
        let instruction = match self {
            QueueNoiseOp::Generate(noise) => ComputeNoiseInstruction {
                images: vec![],
                noise,
                op: NoiseOp::Generator,
            },
            QueueNoiseOp::Modify(input, noise) => {
                let mut images = Vec::new();
                if let Some(input) = input {
                    images.push(input);
                }
                ComputeNoiseInstruction {
                    images,
                    noise,
                    op: NoiseOp::Modifier,
                }
            }
            QueueNoiseOp::Combine(input1, input2, noise) => {
                let mut images = Vec::new();
                if let Some(input1) = input1 {
                    images.push(input1);
                }
                if let Some(input2) = input2 {
                    images.push(input2);
                }
                ComputeNoiseInstruction {
                    images,
                    noise,
                    op: NoiseOp::Combiner,
                }
            }
        };
        ComputeNoiseSequence {
            output,
            instructions: vec![instruction],
        }
    }
}

impl<T: ComputeNoise> IntoNoiseSequence for T {
    fn into_sequence(self, output: Handle<Image>) -> ComputeNoiseSequence {
        QueueNoiseOp::from(self).into_sequence(output)
    }
}

macro_rules! impl_into_noise_sequence_tuple {
    ($($idx:tt : $type:ident),+) => {
        impl<$($type: IntoNoiseSequence),+> IntoNoiseSequence for ($($type,)+) {
            fn into_sequence(self, output: Handle<Image>) -> ComputeNoiseSequence {
                let mut result = ComputeNoiseSequence {
                    output: output.clone(),
                    instructions: Vec::new(),
                };
                $(
                    let seq = self.$idx.into_sequence(output.clone());
                    result.instructions.extend(seq.instructions);
                )+
                result
            }
        }
    };
}

impl_into_noise_sequence_tuple! {0: A}
impl_into_noise_sequence_tuple! {0: A, 1: B}
impl_into_noise_sequence_tuple! {0: A, 1: B, 2: C}
impl_into_noise_sequence_tuple! {0: A, 1: B, 2: C, 3: D}
impl_into_noise_sequence_tuple! {0: A, 1: B, 2: C, 3: D, 4: E}
impl_into_noise_sequence_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F}
impl_into_noise_sequence_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G}
impl_into_noise_sequence_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H}
impl_into_noise_sequence_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I}
impl_into_noise_sequence_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J}
impl_into_noise_sequence_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K}
impl_into_noise_sequence_tuple! {0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L}

// Main World
#[derive(Resource, Default)]
pub struct ComputeNoiseQueue {
    pub(crate) queue: Vec<ComputeNoiseSequence>,
}
impl ComputeNoiseQueue {
    pub fn queue<T: IntoNoiseSequence>(&mut self, output: Handle<Image>, operations: T) {
        self.queue.push(operations.into_sequence(output));
    }
}

pub(crate) fn prepare_compute_noise_buffers(
    images: Res<Assets<Image>>,
    render_device: Res<RenderDevice>,
    mut noise_queue: ResMut<ComputeNoiseQueue>,
    mut noise_buffer_queue: ResMut<ComputeNoiseResourceQueue>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    for item in &noise_queue.queue {
        let sizes: Vec<ComputeNoiseSize> = std::iter::once(&item.output)
            .chain(
                item.instructions
                    .iter()
                    .flat_map(|instruction| instruction.images.iter()),
            )
            .map(|image_handle| {
                images
                    .get(image_handle)
                    .unwrap()
                    .texture_descriptor
                    .size
                    .into()
            })
            .collect();

        if !sizes
            .windows(2)
            .all(|window| TextureDimension::from(window[0]) == TextureDimension::from(window[1]))
        {
            error!("Not all images have the same dimension - did not queue compute noise.");
            continue;
        }

        let buffer = ComputeNoiseResources {
            key: ComputeNoisePipelineKey {
                id: item.key_id(),
                dimension: sizes[0].into(),
            },
            size: sizes[0],
            resources: std::iter::once(ComputeNoiseResource::Image(item.output.clone()))
                .chain(item.instructions.iter().flat_map(|instruction| {
                    instruction
                        .images
                        .iter()
                        .map(|image| ComputeNoiseResource::Image(image.clone()))
                        .chain(
                            instruction
                                .noise
                                .create_buffer(&render_device)
                                .into_iter()
                                .map(ComputeNoiseResource::Buffer),
                        )
                }))
                .collect(),
            shader: shaders.add(Shader::from_wesl(
                generate_noise_shader(sizes[0].into(), item),
                format!("compute_noise_{}.wesl", item.key_id()),
            )),
        };
        noise_buffer_queue.queue.push(buffer);
    }

    noise_queue.queue.clear();
}

#[derive(Clone, Debug)]
pub(crate) enum ComputeNoiseResource {
    Image(Handle<Image>),
    Buffer(Buffer),
}

// Main/Render World
#[derive(Clone)]
pub(crate) struct ComputeNoiseResources {
    pub key: ComputeNoisePipelineKey,
    pub size: ComputeNoiseSize,
    pub resources: Vec<ComputeNoiseResource>,
    pub shader: Handle<Shader>,
}

#[derive(Resource, Clone, Default)]
pub(crate) struct ComputeNoiseResourceQueue {
    pub queue: Vec<ComputeNoiseResources>,
}

// Render World
#[derive(Clone)]
pub struct RenderComputeNoise {
    pub key: ComputeNoisePipelineKey,
    pub size: ComputeNoiseSize,
    pub bind_group: BindGroup,
    pub pipeline_id: CachedComputePipelineId,
}

#[derive(Default, Resource)]
pub(crate) struct ComputeNoiseRenderQueue {
    pub queue: Vec<RenderComputeNoise>,
}
