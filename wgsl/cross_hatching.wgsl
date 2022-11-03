struct InputParams {
    // How close together hatch lines should be placed
    density: f32,
    half_density: f32,
    // How wide hatch lines are drawn.
    width: f32,
    // The brightnesses at which different hatch lines appear
    hatch_1: f32,
    hatch_2: f32,
    hatch_3: f32,
    hatch_4: f32,
    opaque_background_color: f32,
};

///#include "common/group0+vs.wgsl"

///#include "func/edge_detection.wgsl"

// https://www.shadertoy.com/view/MdX3Dr
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
    // frag_color = frag_color * black_edge;
    var alpha = 1.0;
    if (param.opaque_background_color == 0.0) {
        alpha = 1.0 - step(0.3, length(frag_color));
    }

    return vec4<f32>(frag_color, alpha);
}