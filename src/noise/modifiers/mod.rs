use bevy::{app::App, render::render_resource::ShaderRef};

use crate::noise::ComputeNoise;

pub mod invert;
pub use invert::Invert;