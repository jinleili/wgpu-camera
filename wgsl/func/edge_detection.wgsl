fn edge_detection(luminance: f32, step_val: f32) -> f32 {
    return step(step_val, fwidth(luminance));
}