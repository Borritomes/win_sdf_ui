#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

fn grayscale(color: vec4<f32>) -> vec4<f32> {
    return vec4((0.2125 * color.x) + (0.7154 * color.y) + (0.0721 * color.z));
}

fn is_fill(color: vec4<f32>) -> bool {
    return grayscale(color).x >= 0.5;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let dims: vec2<f32> = vec2<f32>(textureDimensions(screen_texture));
    // let uv: vec2<f32> = floor((in.uv * dims) + vec2<f32>(0.5)) / dims;
    // let uv_u32: vec2<u32> = vec2<u32>(dims * uv);
    let uv: vec2<f32> = in.uv;
    let color: vec4<f32> = textureSample(screen_texture, texture_sampler, uv);
    // let color: vec4<f32> = textureLoad(screen_texture, uv_u32, 0);

    if is_fill(grayscale(color)) {
        return vec4<f32>(uv.r, uv.g, 1.0, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}