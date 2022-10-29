#include "base.vs.wgsl"

@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;

#include "func/edge_detection.wgsl"

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(tex, tex_sampler, vertex.uv);
    let gray = length(color.rgb);
    return vec4<f32>(edge_detection(gray, 0.225), 1.0);
}