struct Camera {
    view_matrix: mat4x4<f32>,
    inv_view_matrix: mat4x4<f32>,
    world_pos: vec4<f32>,
}