#define_import_path bevy_compute_noise::perlin

#import bevy_render::maths::PI
#import bevy_compute_noise::util::{random_gradient_2d, random_gradient_3d, interpolate_quintic, interpolate_cubic}

const TILEABLE: u32 = 1u;
const REMAP: u32 = 2u;
const INTERPOLATE_CUBIC: u32 = 4u;

struct Perlin {
    seed: u32,
    frequency: f32,
    flags: u32,
};

fn perlin_2d(uv: vec2<f32>, perlin: Perlin) -> f32 {
    var frequency = perlin.frequency;
    if (perlin.flags & TILEABLE) != 0u { frequency = floor(frequency); }
    let scaled_uv = uv * frequency;

    let grid_id = floor(scaled_uv);
    var grid_uv = fract(scaled_uv);

    let p00 = vec2<u32>(grid_id + vec2<f32>(0.0, 0.0));
    var p10 = vec2<u32>(grid_id + vec2<f32>(1.0, 0.0));
    var p01 = vec2<u32>(grid_id + vec2<f32>(0.0, 1.0));
    var p11 = vec2<u32>(grid_id + vec2<f32>(1.0, 1.0));

    if (perlin.flags & TILEABLE) != 0u {
        p10 = p10 % u32(frequency);
        p01 = p01 % u32(frequency);
        p11 = p11 % u32(frequency);
    }

    let grad00 = random_gradient_2d(perlin.seed, p00);
    let grad10 = random_gradient_2d(perlin.seed, p10);
    let grad01 = random_gradient_2d(perlin.seed, p01);
    let grad11 = random_gradient_2d(perlin.seed, p11);

    let dist00 = grid_uv;
    let dist10 = grid_uv - vec2<f32>(1.0, 0.0);
    let dist01 = grid_uv - vec2<f32>(0.0, 1.0);
    let dist11 = grid_uv - vec2<f32>(1.0, 1.0);

    let dot00 = dot(grad00, dist00);
    let dot10 = dot(grad10, dist10);
    let dot01 = dot(grad01, dist01);
    let dot11 = dot(grad11, dist11);

    if (perlin.flags & INTERPOLATE_CUBIC) != 0u { grid_uv = interpolate_cubic(grid_uv); }
    else { grid_uv = interpolate_quintic(grid_uv); } 

    let b = mix(dot00, dot10, grid_uv.x);
    let t = mix(dot01, dot11, grid_uv.x);

    var value = mix(b, t, grid_uv.y) * sqrt(2.0);

    if (perlin.flags & REMAP) != 0u { value = value * 0.5 + 0.5; }

    return value;
}

fn perlin_3d(uv: vec3<f32>, perlin: Perlin) -> f32 {
    var frequency = perlin.frequency;
    if (perlin.flags & TILEABLE) != 0u { frequency = floor(frequency); }
    let scaled_uv = uv * frequency;

    let grid_id = floor(scaled_uv) % frequency;
    var grid_uv = fract(scaled_uv);

    let p000 = vec3<u32>(grid_id + vec3<f32>(0.0, 0.0, 0.0));
    var p100 = vec3<u32>(grid_id + vec3<f32>(1.0, 0.0, 0.0));
    var p010 = vec3<u32>(grid_id + vec3<f32>(0.0, 1.0, 0.0));
    var p110 = vec3<u32>(grid_id + vec3<f32>(1.0, 1.0, 0.0));
    var p001 = vec3<u32>(grid_id + vec3<f32>(0.0, 0.0, 1.0));
    var p101 = vec3<u32>(grid_id + vec3<f32>(1.0, 0.0, 1.0));
    var p011 = vec3<u32>(grid_id + vec3<f32>(0.0, 1.0, 1.0));
    var p111 = vec3<u32>(grid_id + vec3<f32>(1.0, 1.0, 1.0));

    if (perlin.flags & TILEABLE) != 0u {
        p100 = p100 % u32(frequency);
        p010 = p010 % u32(frequency);
        p110 = p110 % u32(frequency);
        p001 = p001 % u32(frequency);
        p101 = p101 % u32(frequency);
        p011 = p011 % u32(frequency);
        p111 = p111 % u32(frequency);
    }

    let grad000 = random_gradient_3d(perlin.seed, p000);
    let grad100 = random_gradient_3d(perlin.seed, p100);
    let grad010 = random_gradient_3d(perlin.seed, p010);
    let grad110 = random_gradient_3d(perlin.seed, p110);
    let grad001 = random_gradient_3d(perlin.seed, p001);
    let grad101 = random_gradient_3d(perlin.seed, p101);
    let grad011 = random_gradient_3d(perlin.seed, p011);
    let grad111 = random_gradient_3d(perlin.seed, p111);

    let dist000 = grid_uv;
    let dist100 = grid_uv - vec3<f32>(1.0, 0.0, 0.0);
    let dist010 = grid_uv - vec3<f32>(0.0, 1.0, 0.0);
    let dist110 = grid_uv - vec3<f32>(1.0, 1.0, 0.0);
    let dist001 = grid_uv - vec3<f32>(0.0, 0.0, 1.0);
    let dist101 = grid_uv - vec3<f32>(1.0, 0.0, 1.0);
    let dist011 = grid_uv - vec3<f32>(0.0, 1.0, 1.0);
    let dist111 = grid_uv - vec3<f32>(1.0, 1.0, 1.0);

    let dot000 = dot(grad000, dist000);
    let dot100 = dot(grad100, dist100);
    let dot010 = dot(grad010, dist010);
    let dot110 = dot(grad110, dist110);
    let dot001 = dot(grad001, dist001);
    let dot101 = dot(grad101, dist101);
    let dot011 = dot(grad011, dist011);
    let dot111 = dot(grad111, dist111);

    if (perlin.flags & INTERPOLATE_CUBIC) != 0u { grid_uv = interpolate_cubic_3d(grid_uv); }
    else { grid_uv = interpolate_quintic_3d(grid_uv); } 

    let x00 = mix(dot000, dot100, grid_uv.x);
    let x10 = mix(dot010, dot110, grid_uv.x);
    let x01 = mix(dot001, dot101, grid_uv.x);
    let x11 = mix(dot011, dot111, grid_uv.x);

    let y0 = mix(x00, x10, grid_uv.y);
    let y1 = mix(x01, x11, grid_uv.y);

    var value = mix(y0, y1, grid_uv.z) * 1.154701;

    if (perlin.flags & REMAP) != 0u { value = value * 0.5 + 0.5; }

    return value;
}