#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct ThresholdSettings {
    radius: f32,
    threshold: f32,
}
@group(0) @binding(2) var<uniform> settings: ThresholdSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, texture_sampler, in.uv);
    
    var value = 0.5;
    if color.r > settings.threshold {
        value = 1.0;
    } else {
        value = 0.0;
    }

    return vec4(value, value, value, 1.0);
}