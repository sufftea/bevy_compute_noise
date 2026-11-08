use bevy::{asset::Handle, platform::collections::HashSet, render::render_resource::TextureDimension, };

use crate::{noise::ComputeNoise, noise_queue::{ComputeNoiseInstruction, ComputeNoiseSequence, IntoNoiseSequence}, prelude::{Invert, Perlin}, render::pipeline::NoiseOp};

pub trait ComputeNoiseShader {
    fn import_path() -> &'static str;
    fn struct_name() -> Option<&'static str>;
    fn function_2d() -> &'static str;
    fn function_3d() -> &'static str;
}

struct ComputeNoiseShaderBuilder {
    shader: String,
    binding_index: usize,
    dimension: TextureDimension,
}

impl ComputeNoiseShaderBuilder {
    fn new(dimension: TextureDimension) -> Self {
        Self {
            shader: String::new(),
            binding_index: 1,
            dimension,
        }
    }

    fn add_imports(mut self, sequence: &ComputeNoiseSequence) -> Self {
        let mut imports = HashSet::new();
        
        for instruction in &sequence.instructions {
            if imports.insert(instruction.noise.type_id) {
                let fn_name = match self.dimension {
                    TextureDimension::D2 => instruction.noise.function_2d,
                    TextureDimension::D3 => instruction.noise.function_3d,
                    _ => unreachable!(),
                };

                if let Some(struct_name) = instruction.noise.struct_name {
                    self.shader.push_str(&format!(
                        "import {}::{{{}, {}}};\n",
                        instruction.noise.import_path,
                        struct_name,
                        fn_name,
                    ));
                } else {
                    self.shader.push_str(&format!(
                        "import {}::{};\n",
                        instruction.noise.import_path,
                        fn_name,
                    ));
                }
            }
        }
        self.shader.push('\n');
        self
    }

    fn add_bindings(mut self, sequence: &ComputeNoiseSequence) -> Self {
        let dim = match self.dimension {
            TextureDimension::D2 => "2d",
            TextureDimension::D3 => "3d",
            _ => unreachable!("Only 2D and 3D textures are supported"),
        };

        self.shader.push_str(&format!(
            "@group(0) @binding(0) var output_texture: texture_storage_{}<rgba8unorm, read_write>;\n",
            dim,
        ));

        for instruction in &sequence.instructions {
            if instruction.images.len() > 1 {
                for _ in 1..instruction.images.len() {
                    self.shader.push_str(&format!(
                        "@group(0) @binding({}) var input_texture{}: texture_storage_{}<rgba8unorm, read_write>;\n",
                        self.binding_index,
                        self.binding_index,
                        dim,
                    ));
                    self.binding_index += 1;
                }
            }

            if let Some(struct_name) = instruction.noise.struct_name {
                self.shader.push_str(&format!(
                    "@group(0) @binding({}) var<uniform> {}{}: {};\n",
                    self.binding_index,
                    struct_name.to_lowercase(),
                    self.binding_index,
                    struct_name,
                ));
                self.binding_index += 1;
            }
        }
        self
    }

    fn add_main(mut self, sequence: &ComputeNoiseSequence) -> Self {
        self.binding_index = 1;

        let (workgroup, coords, vec_type) = match self.dimension {
            TextureDimension::D2 => ("32, 32", "xy", "vec2"),
            TextureDimension::D3 => ("8, 8, 8", "xyz", "vec3"),
            _ => unreachable!("Only 2D and 3D textures are supported"),
        };

        self.shader.push_str(&format!(
"
@compute @workgroup_size({})
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {{
    let coords = invocation_id.{};
    let texture_size = textureDimensions(output_texture);
    let uv = {}<f32>(coords) / {}<f32>(texture_size);
    var value: f32;
",
            workgroup, coords, vec_type, vec_type
        ));

        for instruction in &sequence.instructions {
            self.add_instruction(instruction);
        }

        self.shader.push_str("    textureStore(output_texture, coords, vec4<f32>(value, 0.0, 0.0, 0.0));\n}");
        self
    }

    fn add_instruction(&mut self, instruction: &ComputeNoiseInstruction) -> &mut Self {
        let fn_name = match self.dimension {
            TextureDimension::D2 => instruction.noise.function_2d,
            TextureDimension::D3 => instruction.noise.function_3d,
            _ => unreachable!("Only 2D and 3D textures are supported"),
        };

        match instruction.op {
            NoiseOp::Generator => {
                if let Some(struct_name) = instruction.noise.struct_name {
                    self.shader.push_str(&format!(
                        "    value = {}(uv, {}{});\n",
                        fn_name,
                        struct_name.to_lowercase(),
                        self.binding_index
                    ));
                    self.binding_index += 1;
                } else {
                    self.shader.push_str(&format!(
                        "    value = {}(uv);\n",
                        fn_name
                    ));
                }
            }
            NoiseOp::Modifier => {
                if instruction.images.len() > 1 {
                    self.shader.push_str(&format!(
                        "\n    let input_size{0} = textureDimensions(input_texture{0});\n    let input_coords{0} = vec2<u32>(vec2<f32>(coords) * vec2<f32>(input_size{0}) / vec2<f32>(texture_size));\n    value = {1}(textureLoad(input_texture{0}, input_coords{0}).r);\n",
                        self.binding_index,
                        fn_name
                    ));
                    self.binding_index += 1;
                } else {
                    self.shader.push_str(&format!(
                        "    value = {}(value);\n",
                        fn_name
                    ));
                }
            },
            _ => {},
        }
        self
    }

    fn build(self) -> String {
        self.shader
    }
}

pub(crate) fn generate_noise_shader(dimension: TextureDimension, sequence: &ComputeNoiseSequence) -> String {
    ComputeNoiseShaderBuilder::new(dimension)
        .add_imports(sequence)
        .add_bindings(sequence)
        .add_main(sequence)
        .build()
}

#[test]
fn test() {
    let sequence: ComputeNoiseSequence = (
        Perlin::default(), 
        Invert.input_image(Handle::default()),
    ).into_sequence(Handle::default());

    generate_noise_shader(TextureDimension::D2, &sequence);
}
