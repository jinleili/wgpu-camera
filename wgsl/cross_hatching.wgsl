///#include "base.vs.wgsl"

@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;

///#include "func/edge_detection.wgsl"

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(tex, tex_sampler, vertex.uv);
    let gray = length(color.rgb);
    let black_edge = vec3<f32>(1.0) - edge_detection(gray, 0.225);

    var frag_color = vec3<f32>(1.0);
    let frag_coord = vertex.position.xy;

    // https://www.shadertoy.com/view/MdX3Dr
    // How close together hatch lines should be placed
    if (gray < 1.0) && ((frag_coord.x + frag_coord.y) % 10.0 == 0.0) {
        frag_color = vec3<f32>(gray);
    } else if (gray < 0.75) && ((frag_coord.x - frag_coord.y) % 10.0 == 0.0) {
        frag_color = vec3<f32>(gray);
    }
     
    if (gray < 0.5) && ((frag_coord.x + frag_coord.y - 5.0) % 10.0  == 0.0) {
        frag_color = vec3<f32>(gray);
    } else if (gray < 0.25) && ((frag_coord.x - frag_coord.y - 5.0) % 10.0  == 0.0) {
        frag_color = vec3<f32>(gray);
    }

    frag_color = min(black_edge, frag_color);

    return vec4<f32>(frag_color, 1.0);
}