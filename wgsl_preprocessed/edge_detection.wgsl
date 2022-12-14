struct InputParams {
    noise_suppression: f32,
    opaque_background_color: f32,
};

struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct UniformData {
    mvp: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> mat_uniform: UniformData;

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) texCoord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = mat_uniform.mvp * vec4<f32>(pos, 1.0);
    out.uv = texCoord;
    return out;
}

@group(0) @binding(1) var<storage> params : array<InputParams>;
@group(0) @binding(2) var tex: texture_2d<f32>;
@group(0) @binding(3) var tex_sampler: sampler;

fn edge_detection(luminance: f32, step_val: f32) -> f32 {
    return step(step_val, fwidth(luminance));
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(tex, tex_sampler, vertex.uv);
    let gray = length(color.rgb);
    let edge = edge_detection(gray, params[0].noise_suppression);

    if (params[0].opaque_background_color >= 1.0) {
        return vec4<f32>(vec3<f32>(edge), 1.0);
    } else {
        return vec4<f32>(vec3<f32>(1.0 - edge), edge);
    }
}
