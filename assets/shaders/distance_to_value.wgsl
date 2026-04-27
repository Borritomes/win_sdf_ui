#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var texture_a: texture_2d<f32>;
@group(0) @binding(1) var texture_b: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;
struct DistanceToValueSettings {
    radius: u32,
}
@group(0) @binding(3) var<uniform> settings: DistanceToValueSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    if in.uv.x < 0.5 {
        let color: vec4<f32> = textureSample(texture_a, texture_sampler, in.uv);
        return color;
    } else {
        let color: vec4<f32> = textureSample(texture_b, texture_sampler, in.uv);
        return color;
    }

    let radius = f32(settings.radius);
    let color: vec4<f32> = textureSample(texture_a, texture_sampler, in.uv);
    let dims = textureDimensions(texture_a);
    let texel_size: vec2<f32> = vec2<f32>(1.0, 1.0) / vec2<f32>(dims);
    let size_of_radius_pixels = vec2<f32>(radius, radius) * texel_size;
    // let uv: vec2<i32> = vec2<i32>(round(in.uv * vec2<f32>(dims)));

    var dist: f32 = distance(in.uv, vec2(color.r, color.g));
    dist = dist;

    // return vec4(dist, dist, dist, 1.0);
    if dist < 0.00125 {
        return vec4(1.0, 1.0, 1.0, 1.0);
    } else {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    return vec4<f32>(color.r, color.g, color.b, color.a);
}