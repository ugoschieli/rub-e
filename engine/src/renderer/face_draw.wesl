import etib::camera::Camera;

struct Face {
    position: vec3<f32>,
    width: u32,
    height: u32,
    direction: u32,
    color: u32,
    _pad: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: u32,
}

const FACE_VERTICES = array<vec3<f32>, 6>(
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 1.0),
    vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 1.0), vec3<f32>(0.0, 0.0, 1.0),
);

/*const FACE_VERTICES = array<vec3<f32>, 6>(
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0),
    vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(0.0, 1.0, 0.0),
);
*/

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> face_matrices: array<mat4x4<f32>, 6>;
@group(0) @binding(2) var<storage, read> faces: array<Face>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let current_face_index: u32 = vertex_index / 6;
    let current_face = faces[current_face_index];
    let current_vertex = FACE_VERTICES[vertex_index % 6];

    let scale = mat4x4<f32>(
        vec4<f32>(f32(current_face.width), 0., 0., 0.),
        vec4<f32>(0.),
        vec4<f32>(0., 0., f32(current_face.height), 0.),
        vec4<f32>(0., 0., 0., 1.0)
    );

    let model_vertex = face_matrices[current_face.direction] * scale * vec4<f32>(current_vertex, 1.);
    let position = vec4<f32>(vec3<f32>(current_face.position), 0.0);
    out.clip_position = camera.view_matrix * (model_vertex + position);
    // out.clip_position = camera * face_matrices[current_face.direction] * vec4<f32>(current_vertex, 1.0);
    out.color = current_face.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = f32(in.color & 1023u) / 1023.0;
    let g = f32((in.color >> 10u) & 1023u) / 1023.0;
    let b = f32((in.color >> 20u) & 1023u) / 1023.0;

    return vec4<f32>(r, g, b, 1.0);
    // return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
