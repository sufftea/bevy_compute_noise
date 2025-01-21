use std::borrow::Cow;

use bevy::{asset::Handle, render::render_resource::ShaderSource, utils::hashbrown::HashSet};

use crate::{noise::ComputeNoise, noise_queue::{ComputeNoiseSequence, IntoNoiseSequence}, prelude::{Invert, Perlin}, render::pipeline::NoiseOp};

pub trait ComputeNoiseShader {
    fn import_path() -> &'static str;
    fn struct_name() -> Option<&'static str>;
    fn function_name() -> &'static str;
}

pub(crate) fn generate_noise_shader(sequence: &ComputeNoiseSequence) -> String {
    let mut shader = String::new();
    add_imports(&mut shader, sequence);
    add_bindings(&mut shader, sequence);
    add_main(&mut shader, sequence);
    println!("{}", &shader);

    shader
}

fn add_imports(shader: &mut String, sequence: &ComputeNoiseSequence) {
    let mut imports = HashSet::new();

    for instruction in &sequence.instructions {
        if imports.insert(instruction.noise.type_id) {
            if let Some(struct_name) = instruction.noise.struct_name {
                shader.push_str(&format!(
                    "#import {}::{{{}, {}}}\n",
                    instruction.noise.import_path,
                    struct_name,
                    instruction.noise.function_name
                ));
            } else {
                shader.push_str(&format!(
                    "#import {}::{}\n",
                    instruction.noise.import_path,
                    instruction.noise.function_name
                ));
            }
        }
    }
    shader.push('\n');
}

fn add_bindings(shader: &mut String, sequence: &ComputeNoiseSequence) {
    shader.push_str(
        "@group(0) @binding(0) var output_texture: texture_storage_2d<rgba8unorm, read_write>;\n"
    );

    let mut binding_index = 1;
    for instruction in &sequence.instructions {
        if instruction.images.len() > 1 {
            for _ in 1..instruction.images.len() {
                shader.push_str(&format!(
                    "@group(0) @binding({}) var input_texture{}: texture_storage_2d<rgba8unorm, read_write>;\n",
                    binding_index,
                    binding_index,
                ));
                binding_index += 1;
            }
        }

        if let Some(struct_name) = instruction.noise.struct_name {
            shader.push_str(&format!(
                "@group(0) @binding({}) var<uniform> {}{}: {};\n",
                binding_index,
                struct_name.to_lowercase(),
                binding_index,
                struct_name,
            ));
            binding_index += 1;
        }
    }
}

fn add_main(shader: &mut String, sequence: &ComputeNoiseSequence) {
    shader.push_str(
"
@compute @workgroup_size(32, 32)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let coords = invocation_id.xy;
    let texture_size = textureDimensions(output_texture);
    let uv = vec2<f32>(coords) / vec2<f32>(texture_size);
    var value: f32;\n
");

    let mut binding_index = 1;
    for instruction in &sequence.instructions {
        match instruction.op {
            NoiseOp::Generator => {
                if let Some(struct_name) = instruction.noise.struct_name {
                    shader.push_str(&format!(
                        "    value = {}(uv, {}{});\n",
                        instruction.noise.function_name,
                        struct_name.to_lowercase(),
                        binding_index
                    ));
                    binding_index += 1;
                } else {
                    shader.push_str(&format!(
                        "    value = {}(uv);\n",
                        instruction.noise.function_name
                    ));
                }
            }
            NoiseOp::Modifier => {
                if instruction.images.len() > 1 {
                    shader.push_str(&format!(
"
    let input_size{0} = textureDimensions(input_texture{0});
    let input_coords{0} = vec2<u32>(vec2<f32>(coords) * vec2<f32>(input_size{0}) / vec2<f32>(texture_size));
    value = {1}(textureLoad(input_texture{0}, input_coords{0}).r);
",
                        binding_index,
                        instruction.noise.function_name,
                    ));
                    binding_index += 1;
                } else {
                    shader.push_str(&format!(
                        "    value = {}(value);\n",
                        instruction.noise.function_name
                    ));
                }
            },
            _ => {},
        }
        
        // Update binding index for input textures
        if instruction.images.len() > 1 {
            binding_index += instruction.images.len() - 1;
        }
    }

    shader.push_str("    textureStore(output_texture, coords, vec4<f32>(value, 0.0, 0.0, 0.0));\n}")
}

#[test]
fn test() {
    let sequence: ComputeNoiseSequence = (
        Perlin::default(), 
        Invert.input_image(Handle::default()),
    ).into_sequence(Handle::default());

    generate_noise_shader(&sequence);
}