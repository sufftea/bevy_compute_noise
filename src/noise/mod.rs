use std::any::{Any, TypeId};

use bevy::{prelude::*, reflect::{FromReflect, GetTypeRegistration, TypePath, Typed}, render::{render_resource::{Buffer, ShaderRef}, renderer::RenderDevice}};
use bytemuck::Pod;

use crate::{noise_queue::QueueNoiseOp, render::pipeline::NoiseOp, shader::ComputeNoiseShader};

pub mod generators;
pub mod modifiers;

pub trait ComputeNoise: Sync + Send + 'static + Default + Clone + TypePath + FromReflect + GetTypeRegistration + Typed + ComputeNoiseShader {
    const NOISE_OP: NoiseOp;

    fn create_buffer(&self, render_device: &RenderDevice) -> Vec<Buffer>;

    fn input_image(self, input: Handle<Image>) -> QueueNoiseOp {
        let erased = ErasedComputeNoise::from(self);
        match Self::NOISE_OP {
            NoiseOp::Modifier => QueueNoiseOp::Modify(Some(input), erased),
            NoiseOp::Combiner => QueueNoiseOp::Combine(Some(input), None, erased),
            _ => panic!("Operation doesn't support input images"),
        }
    }

    fn input_images(self, input1: Handle<Image>, input2: Handle<Image>) -> QueueNoiseOp {
        let erased = ErasedComputeNoise::from(self);
        match Self::NOISE_OP {
            NoiseOp::Combiner => QueueNoiseOp::Combine(Some(input1), Some(input2), erased),
            _ => panic!("Operation doesn't support multiple inputs"),
        }
    }
}

pub trait ComputeNoiseType: ComputeNoise + Pod {
    fn embed_shaders(app: &mut App);
    fn shader() -> ShaderRef;
}

pub struct ErasedComputeNoise {
    noise_data: Box<dyn Any + Send + Sync>,
    buffer_fn: Box<dyn Fn(&RenderDevice) -> Vec<Buffer> + Send + Sync>,
    pub type_id: TypeId,

    pub struct_name: Option<&'static str>,
    pub function_2d: &'static str,
    pub function_3d: &'static str,
    pub import_path: &'static str,
}

impl ErasedComputeNoise {
    pub fn as_noise<T: ComputeNoise>(&self) -> Option<&T> {
        self.noise_data.downcast_ref::<T>()
    }

    pub fn create_buffer(&self, render_device: &RenderDevice) -> Vec<Buffer> {
        (self.buffer_fn)(render_device)
    }

    fn needs_uniform(&self) -> bool {
        self.struct_name.is_some()
    }
}

impl<T: ComputeNoise> From<T> for ErasedComputeNoise {
    fn from(value: T) -> Self {
        Self {
            noise_data: Box::new(value.clone()),
            buffer_fn: Box::new(move |render_device| value.create_buffer(render_device)),
            type_id: TypeId::of::<T>(),

            struct_name: T::struct_name(),
            function_2d: T::function_2d(),
            function_3d: T::function_3d(),
            import_path: T::import_path(),
        }
    }
}