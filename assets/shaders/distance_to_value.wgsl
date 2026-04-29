#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var texture_regular: texture_2d<f32>;
@group(0) @binding(1) var texture_invert: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;
struct DistanceFieldSettings {
    threshold: f32,
    radius: f32,
}
@group(0) @binding(3) var<uniform> settings: DistanceFieldSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let radius = f32(settings.radius);
    let color_regular: vec4<f32> = textureSample(texture_regular, texture_sampler, in.uv);
    let color_invert: vec4<f32> = textureSample(texture_invert, texture_sampler, in.uv);

    let dims = textureDimensions(texture_regular);
    let texel_size: vec2<f32> = vec2<f32>(1.0, 1.0) / vec2<f32>(dims);
    let size_of_radius_pixels = vec2<f32>(radius, radius) * texel_size;
    let uv: vec2<i32> = vec2<i32>(round(in.uv * vec2<f32>(dims)));

    var dist_regular: f32 = distance(in.uv, vec2(color_regular.r, color_regular.g));
    var dist_invert: f32 = distance(in.uv, vec2(color_invert.r, color_invert.g));

    if false {
        if color_regular.b > 0.5 {
            let dist = distance(vec2<f32>(uv), round(vec2(color_invert.r, color_invert.g) * vec2<f32>(dims)));
            if dist > radius {
                return vec4(1.0, 1.0, 1.0, 1.0);
            }
            let val = 0.5 + ((1.0 / radius) * dist);
            return vec4(val, val, val, 1.0);
        } else {
            let dist = distance(vec2<f32>(uv), round(vec2(color_regular.r, color_regular.g) * vec2<f32>(dims)));
            let val = 0.5 - ((1.0 / radius) * dist);
            return vec4(val, val, val, 1.0);
        }
    }

    // return vec4(dist_regular, dist_invert, color_regular.b, 1.0);
    if false {
        //distance field gradient
        if color_regular.b > 0.5 {
            // inside so use inverted
            let dist = 0.5 + (dist_invert / 2.0);
            return vec4(dist, dist, dist, 1.0);
        } else {
            // outside so use regular
            let dist = 0.5 - (dist_regular / 2);
            return vec4(dist, dist, dist, 1.0);
        }
    } else {
        // thresholded
        var dist = 0.0;
        if color_regular.b > 0.5 {
            // inside so use inverted
            dist = 0.5 + (dist_invert / 2.0);
        } else {
            // outside so use regular
            dist = 0.5 - (dist_regular / 2);
        }

        if dist > 0.5 {
            return vec4(1.0);
        } else {
            return vec4(0.0, 0.0, 0.0, 1.0);
        }
    }
}