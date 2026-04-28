// generates a distance field using the jump-flood algorithm
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
// I dont think this is needed :P
struct DistanceFieldSettings {
    radius: u32,
}
@group(0) @binding(2) var<uniform> settings: DistanceFieldSettings;
@group(0) @binding(3) var<uniform> step_size: u32;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let dims: vec2<f32> = vec2<f32>(textureDimensions(screen_texture));
    let uv: vec2<i32> = vec2<i32>(floor(
        (in.uv * vec2<f32>(dims)) + vec2(0.5, 0.5)
    ));
    let step_size_i32 = i32(step_size);
    let color = textureLoad(screen_texture, uv, 0);

    // if color.r == in.uv.r && color.g == in.uv.g {
    //     return vec4(1.0);
    // } else {
    //     return vec4(0.0, 0.0, 0.0, 1.0);
    // }

    var best_sample: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var best_distance: f32 = 99999999.0;

    var b = 0.25;

    for (var x: i32 = -1; x <= 1; x++) {
        for (var y: i32 = -1; y <= 1; y++) {
            let offset: vec2<i32> = vec2<i32>(x, y) * vec2<i32>(step_size_i32);
            let sample_pos: vec2<i32> = uv + offset;
            let sample: vec4<f32> = textureLoad(screen_texture, sample_pos, 0);

            if sample.a == 0.0 {
                continue;
            }

            let dist = distance(in.uv, sample.xy);
            if dist < best_distance {
                best_distance = dist;
                best_sample = vec4(sample.r, sample.g, color.b, 1.0);
            }
        }
    }

    // best_sample.b = 0.0;
    return best_sample;
}