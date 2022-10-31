struct InputParams {
    temp: f32,
};

///#include "common/group0+vs.wgsl"

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, tex_sampler, vertex.uv);
}