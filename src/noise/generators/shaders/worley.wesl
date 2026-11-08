// #define_import_path bevy_compute_noise::worley

import bevy_compute_noise::noise::shaders::util::{hash22, hash33, INFINITY};

const TILEABLE: u32 = 1u;

struct Worley {
    seed: u32,
    frequency: f32,
    flags: u32,
};

fn worley_2d(uv: vec2<f32>, worley: Worley) -> f32 {
    let frequency = worley.frequency;
    let scaled_uv = uv * frequency;
    
    let cell_id = floor(scaled_uv);
    let local_pos = fract(scaled_uv);
    
    var min_distance = INFINITY;
    for (var x: i32 = -1; x <= 1; x++) {
        for (var y: i32 = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y));
            
            let id = select(
                cell_id + vec2<f32>(f32(x), f32(y)),
                vec2<f32>(
                    fract((cell_id.x + f32(x)) / frequency) * frequency,
                    fract((cell_id.y + f32(y)) / frequency) * frequency
                ),
                (worley.flags & TILEABLE) != 0u
            );

            let seeded_id = id + vec2<f32>(f32(worley.seed) * 333, f32(worley.seed) * 563);
            
            let h = (hash22(seeded_id) * 0.5 + 0.5);
            let point_pos = offset + h;
            
            let d = local_pos - point_pos;
            min_distance = min(min_distance, length(d));
        }
    }

    var value = min_distance;

    return value;
}

fn worley_3d(uv: vec3<f32>, worley: Worley) -> f32 {
    let frequency = worley.frequency;
    let scaled_uv = uv * frequency;
    
    let cell_id = floor(scaled_uv);
    let local_pos = fract(scaled_uv);

    var min_distance = INFINITY;
    for (var x: i32 = -1; x <= 1; x++) {
        for (var y: i32 = -1; y <= 1; y++) {
            for (var z: i32 = -1; z <= 1; z++) {
                let offset = vec3<f32>(f32(x), f32(y), f32(z));
            
                let id = select(
                    cell_id + vec3<f32>(f32(x), f32(y), f32(z)),
                    vec3<f32>(
                        fract((cell_id.x + f32(x)) / frequency) * frequency,
                        fract((cell_id.y + f32(y)) / frequency) * frequency,
                        fract((cell_id.z + f32(z)) / frequency) * frequency
                    ),
                    (worley.flags & TILEABLE) != 0u
                );

                let seeded_id = id + vec3<f32>(f32(worley.seed) * 333, f32(worley.seed) * 563, f32(worley.seed) * 122);
                
                let h = (hash33(seeded_id) * 0.5 + 0.5);
                let point_pos = offset + h;
                
                let d = local_pos - point_pos;
                min_distance = min(min_distance, length(d));
            }
        }
    }

    // var normalized_distance = min_distance / distance(vec3<f32>(0.0, 0.0, 0.0), cell_size);

    var value = min_distance;

    return value;
}
