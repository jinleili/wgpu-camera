struct InputParams {
    ascii_width_uv: vec2<f32>,
    ascii_width: f32,
    half_aw: f32,
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

fn character(n: i32, p: vec2<f32>) -> f32 {
	let np = floor(p * vec2<f32>(4.0, -4.0) + 2.5);
    if (clamp(np.x, 0.0, 4.0) == np.x) && (clamp(np.y, 0.0, 4.0) == np.y) {
        let a = u32(round(np.x) + 5.0 * round(np.y));
        if (((n >> a) & 1) == 1) {
            return 1.0;
        }
    }
	return 0.0;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let params = params[0];
    let uv = floor(vertex.uv / params.ascii_width_uv) * params.ascii_width_uv;
    let color = textureSample(tex, tex_sampler, uv).rgb;
    let gray = length(color);

    var n: i32 =  4096;               // .
	if (gray > 0.2) { n = 65600; }    // :
    if (gray > 0.3) { n = 22483413; } // *
	if (gray > 0.4) { n = 15255086; } // o 
    if (gray > 0.5) { n = 4357252; }  // +
	if (gray > 0.6) { n = 15252014; } // 8
    if (gray > 0.7) { n = 4532799; }  // ∆
	if (gray > 0.8) { n = 11512810; } // #
	
    let p = ((vertex.position.xy / params.half_aw) % 2.0) - vec2<f32>(1.0);
    return vec4<f32>(color * character(n, p), 1.0);
}
