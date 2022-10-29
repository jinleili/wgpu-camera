///#include "base.vs.wgsl"

@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;

///#include "func/edge_detection.wgsl"

let threshold_1 = 0.95;
let threshold_2 = 0.7;
let threshold_3 = 0.5;
let threshold_4 = 0.2;
// How close together hatch lines should be placed
let density = 10.0;
let half_density = 5.0;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(tex, tex_sampler, vertex.uv);
    let gray = length(color.rgb);
    let black_edge = vec3<f32>(1.0) - edge_detection(gray, 0.225);

    var frag_color = vec3<f32>(1.0);
    let frag_coord = vertex.position.xy;

    // https://www.shadertoy.com/view/MdX3Dr
    if (gray < threshold_1) && ((frag_coord.x + frag_coord.y) % density == 0.0) {
        frag_color = vec3<f32>(gray);
    } else if (gray < threshold_2) && ((frag_coord.x - frag_coord.y) % density == 0.0) {
        frag_color = vec3<f32>(gray);
    }
     
    if (gray < threshold_3) && ((frag_coord.x + frag_coord.y - half_density) % density  == 0.0) {
        frag_color = vec3<f32>(gray);
    } else if (gray < threshold_4) && ((frag_coord.x - frag_coord.y - half_density) % density  == 0.0) {
        frag_color = vec3<f32>(gray);
    }

    frag_color = min(black_edge, frag_color);

    return vec4<f32>(frag_color, 1.0);
}