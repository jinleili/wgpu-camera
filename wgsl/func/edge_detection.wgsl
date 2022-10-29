fn edge_detection(luminance: f32, step_val: f32) -> vec3<f32> {
    return vec3<f32>(step(step_val, fwidth(luminance)));
}