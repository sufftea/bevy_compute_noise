use bevy::{
    material::descriptor::BindGroupLayoutDescriptor,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    render::{
        render_resource::{
            binding_types::{texture_storage_2d, uniform_buffer_sized},
            BindGroupLayout, BindGroupLayoutEntries, BindGroupLayoutEntryBuilder, BindingType,
            ComputePipelineDescriptor, IntoBindGroupLayoutEntryBuilder, ShaderStages,
            SpecializedComputePipeline, StorageTextureAccess, TextureDimension, TextureFormat,
            TextureViewDimension,
        },
        renderer::RenderDevice,
    },
    shader::ShaderRef,
};

use crate::noise::{generators::Fbm, ComputeNoiseType};

// pub fn load_generator_shader<T: ComputeNoiseGenerator>(world: &mut World) {
//     let shader_2d = match T::shader_2d() {
//         ShaderRef::Default => None,
//         ShaderRef::Handle(handle) => Some(handle),
//         ShaderRef::Path(path) => Some(world.resource::<AssetServer>().load(path)),
//     }.unwrap();

//     let shader_3d = match T::shader_3d() {
//         ShaderRef::Default => None,
//         ShaderRef::Handle(handle) => Some(handle),
//         ShaderRef::Path(path) => Some(world.resource::<AssetServer>().load(path)),
//     }.unwrap();

//     let mut pipeline = world.resource_mut::<ComputeNoisePipeline>();

//     pipeline.shaders.insert(
//         ComputeNoisePipelineKey {
//             type_id: TypeId::of::<T>(),
//             dimension: TextureDimension::D2,
//             op: NoiseOp::Generator,
//         },
//         shader_2d,
//     );

//     pipeline.shaders.insert(
//         ComputeNoisePipelineKey {
//             type_id: TypeId::of::<T>(),
//             dimension: TextureDimension::D3,
//             op: NoiseOp::Generator,
//         },
//         shader_3d,
//     );
// }

// pub fn load_fbm_shaders<T: ComputeNoiseGenerator>(world: &mut World) {
//     let shader = world.resource::<AssetServer>().load("embedded://bevy_compute_noise/noise/generators/shaders/fbm.wgsl");
//     let mut pipeline = world.resource_mut::<ComputeNoisePipeline>();

//     pipeline.shaders.insert(
//         ComputeNoisePipelineKey {
//             type_id: TypeId::of::<Fbm<T>>(),
//             dimension: TextureDimension::D2,
//             op: NoiseOp::Generator,
//         },
//         shader.clone(),
//     );

//     pipeline.shaders.insert(
//         ComputeNoisePipelineKey {
//             type_id: TypeId::of::<Fbm<T>>(),
//             dimension: TextureDimension::D3,
//             op: NoiseOp::Generator,
//         },
//         shader,
//     );

//     pipeline.shader_defs.insert(
//         ComputeNoisePipelineKey {
//             type_id: TypeId::of::<Fbm<T>>(),
//             dimension: TextureDimension::D2,
//             op: NoiseOp::Generator,
//         },
//         vec![T::shader_def(), "2D".into()]
//     );
//     pipeline.shader_defs.insert(
//         ComputeNoisePipelineKey {
//             type_id: TypeId::of::<Fbm<T>>(),
//             dimension: TextureDimension::D3,
//             op: NoiseOp::Generator,
//         },
//         vec![T::shader_def(), "3D".into()]
//     );
// }

// pub fn load_modifier_shader<T: ComputeNoiseModifier>(world: &mut World) {
//     let shader = match T::shader() {
//         ShaderRef::Default => None,
//         ShaderRef::Handle(handle) => Some(handle),
//         ShaderRef::Path(path) => Some(world.resource::<AssetServer>().load(path)),
//     }.unwrap();

//     let mut pipeline = world.resource_mut::<ComputeNoisePipeline>();
//      pipeline.shaders.insert(
//         ComputeNoisePipelineKey {
//             type_id: TypeId::of::<T>(),
//             dimension: TextureDimension::D2,
//             op: NoiseOp::Modifier,
//         },
//         shader.clone(),
//     );

//     pipeline.shaders.insert(
//         ComputeNoisePipelineKey {
//             type_id: TypeId::of::<T>(),
//             dimension: TextureDimension::D3,
//             op: NoiseOp::Modifier,
//         },
//         shader,
//     );

//     pipeline.shader_defs.insert(
//         ComputeNoisePipelineKey {
//             type_id: TypeId::of::<T>(),
//             dimension: TextureDimension::D2,
//             op: NoiseOp::Modifier,
//         },
//         vec!["2D".into()]
//     );
//     pipeline.shader_defs.insert(
//         ComputeNoisePipelineKey {
//             type_id: TypeId::of::<T>(),
//             dimension: TextureDimension::D3,
//             op: NoiseOp::Modifier,
//         },
//         vec!["3D".into()]
//     );
// }

pub(crate) fn load_base_shader<T: ComputeNoiseType>(world: &mut World) {
    let shader = match T::shader() {
        ShaderRef::Default => None,
        ShaderRef::Handle(handle) => Some(handle),
        ShaderRef::Path(path) => Some(world.resource::<AssetServer>().load(path)),
    }
    .unwrap();

    let mut pipeline = world.resource_mut::<ComputeNoisePipeline>();
    pipeline.base_shaders.insert(shader);
}

#[derive(Resource)]
pub struct ComputeNoisePipeline {
    pub base_shaders: HashSet<Handle<Shader>>,
    pub layouts: HashMap<ComputeNoisePipelineKey, BindGroupLayoutDescriptor>,
    pub shaders: HashMap<ComputeNoisePipelineKey, Handle<Shader>>,
    _util_shader: Handle<Shader>,
}

impl FromWorld for ComputeNoisePipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            base_shaders: HashSet::new(),
            layouts: HashMap::new(),
            shaders: HashMap::new(),
            _util_shader: world
                .resource::<AssetServer>()
                .load("embedded://bevy_compute_noise/noise/shaders/util.wesl"),
        }
    }
}

impl ComputeNoisePipeline {
    pub fn get_layout_descriptor(
        &self,
        key: ComputeNoisePipelineKey,
    ) -> &BindGroupLayoutDescriptor {
        self.layouts.get(&key).unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ComputeNoisePipelineKey {
    pub id: u64,
    pub dimension: TextureDimension,
}

impl SpecializedComputePipeline for ComputeNoisePipeline {
    type Key = ComputeNoisePipelineKey;

    fn specialize(&self, key: Self::Key) -> ComputePipelineDescriptor {
        let layout_descriptor = self.get_layout_descriptor(key);

        ComputePipelineDescriptor {
            label: Some("compute_noise_pipeline".into()),
            layout: vec![layout_descriptor.clone()],
            // push_constant_ranges: Vec::new(),
            shader: self.shaders.get(&key).unwrap().clone(),
            shader_defs: Vec::new(),
            entry_point: Some("main".into()),
            zero_initialize_workgroup_memory: false,
            ..Default::default()
        }
    }
}

pub fn noise_texture_2d() -> BindGroupLayoutEntryBuilder {
    texture_storage_2d(TextureFormat::Rgba8Unorm, StorageTextureAccess::ReadWrite)
}

pub fn noise_texture_3d() -> BindGroupLayoutEntryBuilder {
    BindingType::StorageTexture {
        access: StorageTextureAccess::ReadWrite,
        format: TextureFormat::Rgba8Unorm,
        view_dimension: TextureViewDimension::D3,
    }
    .into_bind_group_layout_entry_builder()
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum NoiseOp {
    Generator,
    Modifier,
    Combiner,
}

