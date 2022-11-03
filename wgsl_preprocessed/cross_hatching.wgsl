struct InputParams {
    density: f32,
    half_density: f32,
    width: f32,
    hatch_1: f32,
    hatch_2: f32,
    hatch_3: f32,
    hatch_4: f32,
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
    let black_edge = 1.0 - edge_detection(gray, 0.125);

    var frag_color = vec3<f32>(1.0);
    let frag_coord = vertex.position.xy;

    let param = params[0];
    if (gray < param.hatch_1) && ((frag_coord.x + frag_coord.y) % param.density <= param.width) {
        frag_color = vec3<f32>(gray);
    }
    if (gray < param.hatch_2) && (abs((frag_coord.x - frag_coord.y)) % param.density <= param.width) {
        frag_color = vec3<f32>(gray);
    }
     
    if (gray < param.hatch_3) && (abs((frag_coord.x + frag_coord.y - param.half_density)) % param.density <= param.width) {
        frag_color = vec3<f32>(gray);
    }
    if (gray < param.hatch_4) && (abs((frag_coord.x - frag_coord.y - param.half_density)) % param.density <= param.width) {
        frag_color = vec3<f32>(0.0);
    }

    frag_color = min(vec3<f32>(black_edge), frag_color);
    var alpha = 1.0;
    if (param.opaque_background_color == 0.0) {
        alpha = 1.0 - step(0.3, length(frag_color));
    }

    return vec4<f32>(frag_color, alpha);
}
