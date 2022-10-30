///#include "base.vs.wgsl"

@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;

let ascii_width = 8.0;
let half_aw = 4.0;

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
    let pix = vertex.position.xy;
    let uv = floor(vertex.position.xy/ascii_width)*ascii_width/vec2<f32>(512.);
    let color = textureSample(tex, tex_sampler, uv).rgb;
    // let color = textureSample(tex, tex_sampler, vertex.uv).rgb;
    let gray = length(color);

    var n: i32 =  4096;               // .
	if (gray > 0.2) { n = 65600; }    // :
	if (gray > 0.3) { n = 332772; }   // *
	if (gray > 0.4) { n = 15255086; } // o 
	if (gray > 0.5) { n = 23385164; } // &
	if (gray > 0.6) { n = 15252014; } // 8
	if (gray > 0.7) { n = 13199452; } // @
	if (gray > 0.8) { n = 11512810; } // #
	
    let p = ((pix/half_aw) % 2.0) - vec2<f32>(1.0);
    return vec4<f32>(color * character(n, p), 1.0);
}