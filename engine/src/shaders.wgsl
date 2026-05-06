struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

const CUBE_VERTICES = array<vec3<f32>, 6>(
    vec3<f32>(-0.5, -0.5, 0.0), vec3<f32>(0.5, -0.5, 0.0), vec3<f32>(0.5, 0.5, 0.0),
    vec3<f32>(0.5, 0.5, 0.0), vec3<f32>(-0.5, 0.5, 0.0), vec3<f32>(-0.5, -0.5, 0.0),
);

@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let current_vertex = CUBE_VERTICES[vertex_index];
    out.clip_position = camera * vec4<f32>(current_vertex, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
