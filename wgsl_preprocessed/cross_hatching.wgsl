struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct MVPMatUniform {
    mvp: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> mat_uniform: MVPMatUniform;

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

@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;

fn edge_detection(luminance: f32, step_val: f32) -> vec3<f32> {
    return vec3<f32>(step(step_val, fwidth(luminance)));
}

let hatch_y_offset = 5.0;
let threshold_1 = 0.95;
let threshold_2 = 0.7;
let threshold_3 = 0.5;
let threshold_4 = 0.2;

let density = 10.0;
let half_density = 5.0;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(tex, tex_sampler, vertex.uv);
    let gray = length(color.rgb);
    let black_edge = vec3<f32>(1.0) - edge_detection(gray, 0.225);

    var frag_color = vec3<f32>(1.0);
    let frag_coord = vertex.position.xy;
    if (gray < threshold_1) && ((frag_coord.x + frag_coord.y) % density == 0.0) {
            frag_color = vec3<f32>(gray);
    } else if (gray < threshold_2) && ((frag_coord.x - frag_coord.y) % density == 0.0) {
        frag_color = vec3<f32>(gray);
    }
     
    if (gray < threshold_3) {
        if ((frag_coord.x + frag_coord.y - half_density) % density  == 0.0) {
            frag_color = vec3<f32>(gray);
        }
    }
     
    if (gray < threshold_4) {
        if ((frag_coord.x - frag_coord.y - half_density) % density  == 0.0) {
            frag_color = vec3<f32>(gray);
        }
    }

    frag_color = min(black_edge, frag_color);

    return vec4<f32>(frag_color, 1.0);
}
