use bevy::{
    asset::embedded_asset, prelude::*, render::{
        render_resource::{Buffer, BufferInitDescriptor, BufferUsages},
        renderer::RenderDevice,
    }, shader::ShaderRef,
};
use bytemuck::{Pod, Zeroable};

use crate::{
    noise::{ComputeNoise, ComputeNoiseType},
    render::pipeline::NoiseOp,
    shader::ComputeNoiseShader,
};

#[derive(Clone, Copy, Reflect, PartialEq, Debug, Pod, Zeroable)]
#[reflect(Default)]
#[repr(C)]
pub struct Worley {
    pub seed: u32,
    pub frequency: f32,
    pub flags: u32,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct WorleyFlags: u32 {
        const TILEABLE = 1 << 0;
    }
}

impl Default for Worley {
    fn default() -> Self {
        Self {
            seed: 0,
            frequency: 5.0,
            flags: 0,
        }
    }
}

impl ComputeNoise for Worley {
    const NOISE_OP: NoiseOp = NoiseOp::Generator;

    fn create_buffer(&self, render_device: &RenderDevice) -> Vec<Buffer> {
        vec![
            render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("worley2d_points_buffer"),
                contents: &bytemuck::cast_slice(&[self.clone()]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }),
        ]
    }
}

impl ComputeNoiseType for Worley {
    fn shader() -> ShaderRef {
        "embedded://bevy_compute_noise/noise/generators/shaders/worley.wesl".into()
    }

    fn embed_shaders(app: &mut App) {
        embedded_asset!(app, "shaders/worley.wesl");
    }
}

impl ComputeNoiseShader for Worley {
    fn function_2d() -> &'static str {
        "worley_2d"
    }
    fn function_3d() -> &'static str {
        "worley_3d"
    }

    fn import_path() -> &'static str {
        "bevy_compute_noise::noise::generators::shaders::worley"
    }

    fn struct_name() -> Option<&'static str> {
        Some("Worley")
    }
}

