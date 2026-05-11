struct Cube {
    position: vec3<i32>,
    color: u32,
}

struct Face {
    position: vec3<i32>,
    direction: u32,
    color: u32,
}

@group(0) @binding(0) var<storage, read> cubes: array<Cube>;
@group(0) @binding(1) var<storage, read_write> faces: array<Face>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let cube_index = id.x;

    let cubes_len = arrayLength(&cubes);
    if cube_index >= cubes_len {
        return;
    }

    let cube = cubes[cube_index];

    let base_face_index = cube_index * 6u;
    for (var i = 0u; i < 6u; i++) {
        faces[base_face_index + i] = Face(cube.position, i, cube.color);
    }
}