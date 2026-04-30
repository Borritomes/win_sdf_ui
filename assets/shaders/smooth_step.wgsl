#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct SmoothStepSettings {
    falloff_start: f32,
    falloff_stop: f32
}
@group(0) @binding(2) var<uniform> settings: SmoothStepSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color: vec4<f32> = textureSample(screen_texture, texture_sampler, in.uv);
    let settings = SmoothStepSettings(0.4, 0.6);
    let uv = vec2<i32>(vec2<f32>(textureDimensions(screen_texture)) * in.uv);

    var feathered: f32 = smoothstep(settings.falloff_start, settings.falloff_stop, color.r);

    if feathered > 0.5 {
        feathered = 1.0;
    } else {
        feathered = 0.0;
    }

    return vec4(vec3(feathered), 1.0);
}