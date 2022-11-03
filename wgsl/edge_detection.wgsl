struct InputParams {
    noise_suppression: f32,
    opaque_background_color: f32,
};

///#include "common/group0+vs.wgsl"

///#include "func/edge_detection.wgsl"

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(tex, tex_sampler, vertex.uv);
    let gray = length(color.rgb);
    // let gray = (0.2126*color.r) + (0.7152*color.g) + (0.0722*color.b);
    let edge = edge_detection(gray, params[0].noise_suppression);

    if (params[0].opaque_background_color >= 1.0) {
        return vec4<f32>(vec3<f32>(edge), 1.0);
    } else {
        return vec4<f32>(vec3<f32>(1.0 - edge), edge);
    }
}