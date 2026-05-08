struct Cube {
    position: vec4<f32>,
    color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

const FACE_VERTICES = array<vec3<f32>, 6>(
    vec3<f32>(-0.5, -0.5, 0.0), vec3<f32>(0.5, -0.5, 0.0), vec3<f32>(0.5, 0.5, 0.0),
    vec3<f32>(0.5, 0.5, 0.0), vec3<f32>(-0.5, 0.5, 0.0), vec3<f32>(-0.5, -0.5, 0.0),
);

@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;
@group(0) @binding(1)
var<storage, read> cubes: array<Cube>;
@group(0) @binding(2)
var<storage, read> face_matrices: array<mat4x4<f32>, 6>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let current_cube: u32 = vertex_index / 36;
    let current_face: u32 = (vertex_index / 6) % 6;
    let current_vertex = FACE_VERTICES[vertex_index % 6];

    let model_vertex = face_matrices[current_face] * vec4<f32>(current_vertex, 1.);
    let position: vec4<f32> = cubes[current_cube].position;
    out.clip_position = camera * (model_vertex + position);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
