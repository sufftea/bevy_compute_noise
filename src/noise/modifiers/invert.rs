use bevy::{app::App, asset::embedded_asset, reflect::Reflect, render::{render_resource::Buffer, renderer::RenderDevice}, shader::ShaderRef};
use bytemuck::{Pod, Zeroable};

use crate::{noise::{ComputeNoise, ComputeNoiseType}, render::pipeline::NoiseOp, shader::ComputeNoiseShader};


#[derive(Clone, Copy, Reflect, Default, Pod, Zeroable)]
#[repr(C)]
pub struct Invert;

impl ComputeNoise for Invert {
    const NOISE_OP: NoiseOp = NoiseOp::Modifier; 

    fn create_buffer(&self, _render_device: &RenderDevice) -> Vec<Buffer> {
        Vec::new()
    }
}

impl ComputeNoiseType for Invert {
    fn embed_shaders(app: &mut App) {     
        embedded_asset!(app, "shaders/invert.wesl");
    }

    fn shader() -> ShaderRef {
       "embedded://bevy_compute_noise/noise/modifiers/shaders/invert.wesl".into() 
    }
}

impl ComputeNoiseShader for Invert {
    fn function_2d() -> &'static str {
        "invert"
    }
    fn function_3d() -> &'static str {
        "invert"
    }

    fn import_path() -> &'static str {
        "bevy_compute_noise::noise::modifiers::shaders::invert"
    }

    fn struct_name() -> Option<&'static str> {
        None
    }
}
