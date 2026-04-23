#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct DistanceFieldSettings {
    radius: u32,
}
@group(0) @binding(2) var<uniform> settings: DistanceFieldSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let radius = f32(settings.radius);
    let color: vec4<f32> = textureSample(screen_texture, texture_sampler, in.uv);
    let dims = textureDimensions(screen_texture);
    let texel_size: vec2<f32> = vec2<f32>(1.0, 1.0) / vec2<f32>(dims);
    let size_of_radius_pixels = vec2<f32>(radius, radius) * texel_size;
    // let uv: vec2<i32> = vec2<i32>(round(in.uv * vec2<f32>(dims)));

    var dist: f32 = distance(in.uv, vec2(color.r, color.g));
    dist = dist;

    if in.uv.x > 0.5 {
        return vec4(dist, dist, dist, 1.0);
    }
    if dist <= 0.001 {
        return vec4(1.0, 1.0, 1.0, 1.0);
    } else {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    return vec4<f32>(color.r, color.g, color.b, color.a);
}