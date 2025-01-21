use bevy::{prelude::*, render::render_resource::{ShaderDefVal, ShaderRef}};

pub mod worley;
pub mod perlin;
pub mod fbm;

use bytemuck::Pod;
pub use worley::{Worley, WorleyFlags};
pub use perlin::{Perlin, PerlinFlags};
pub use fbm::Fbm;