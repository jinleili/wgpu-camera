struct InputParams {
    temp: f32,
};

///#include "common/group0+vs.wgsl"

///#include "func/edge_detection.wgsl"

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(tex, tex_sampler, vertex.uv);
    let gray = length(color.rgb);
    // let gray = (0.2126*color.r) + (0.7152*color.g) + (0.0722*color.b);

    return vec4<f32>(edge_detection(gray, 0.135), 1.0);
}