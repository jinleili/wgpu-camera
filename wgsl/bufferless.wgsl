struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertexIndex: u32) -> VertexOutput {
    let uv: vec2<f32> = vec2<f32>(f32((vertexIndex << 1u) & 2u), f32(vertexIndex & 2u));
    var result: VertexOutput;
    result.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    // invert uv.y
    result.uv = vec2<f32>(uv.x, (uv.y - 1.0) *  (-1.0));
    return result;
}

@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(tex, tex_sampler, vertex.uv);
    let gray = length(color.rgb);
    return vec4<f32>(vec3<f32>(step(0.25, fwidth(gray))), 1.0);
}