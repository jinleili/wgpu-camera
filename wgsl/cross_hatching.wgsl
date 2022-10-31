struct InputParams {
    // How close together hatch lines should be placed
    density: f32,
    half_density: f32,
    // How wide hatch lines are drawn.
    width: f32,

};

///#include "common/group0+vs.wgsl"

///#include "func/edge_detection.wgsl"

// https://www.shadertoy.com/view/MdX3Dr
@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(tex, tex_sampler, vertex.uv);
    let gray = length(color.rgb);
    let black_edge = vec3<f32>(1.0) - edge_detection(gray, 0.225);

    var frag_color = vec3<f32>(1.0);
    let frag_coord = vertex.position.xy;

    let params = params[0];
    if (gray < 1.0) && ((frag_coord.x + frag_coord.y) % params.density <= params.width) {
        frag_color = vec3<f32>(gray);
    }
    if (gray < 0.75) && (abs((frag_coord.x - frag_coord.y) % params.density) <= params.width) {
        frag_color = vec3<f32>(gray);
    }
     
    if (gray < 0.5) && ((frag_coord.x + frag_coord.y - params.half_density) % params.density <= params.width) {
        frag_color = vec3<f32>(gray);
    }
    if (gray < 0.25) && (abs((frag_coord.x - frag_coord.y - params.half_density) % params.density) <= params.width) {
        frag_color = vec3<f32>(gray);
    }

    // frag_color = min(black_edge, frag_color);
    frag_color = frag_color * black_edge;

    return vec4<f32>(frag_color, 1.0);
}