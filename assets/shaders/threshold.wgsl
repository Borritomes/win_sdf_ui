#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct ThresholdSettings {
    threshold: f32,
}
@group(0) @binding(2) var<uniform> settings: ThresholdSettings;
@group(0) @binding(3) var<uniform> invert: u32;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, texture_sampler, in.uv);
    
    // yellow vec4(1.0, 0.6, 0.0, 1.0)
    // falsecase, truecase, condition
    let fill: f32 = select(0.0, 1.0, invert < 1);

    var value = 0.5;
    if color.r > settings.threshold {
        value = fill;
    } else {
        value = 1.0 - fill;
    }

    return vec4(value, value, value, 1.0);
}