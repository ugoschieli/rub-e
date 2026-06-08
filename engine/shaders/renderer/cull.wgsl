const CHUNK_SIZE: f32 = 32.0;

struct ChunkMeta {
    position: vec3<i32>,
    offset: u32,
}

// count is at offset 0, indices follow immediately after.
// Allocate the buffer as: sizeof(u32) + num_chunks * sizeof(u32).
struct VisibleChunks {
    count: atomic<u32>,
    indices: array<u32>,
}

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(1) var<storage, read> chunks: array<ChunkMeta>;
@group(0) @binding(2) var<storage, read_write> visible_chunks: VisibleChunks;

// Extract 5 frustum planes via Gribb-Hartmann from the view-projection matrix.
// The far plane is omitted because the camera uses an infinite projection.
// Plane equation: dot(plane.xyz, world_pos) + plane.w >= 0 means inside.
fn extract_planes(m: mat4x4<f32>) -> array<vec4<f32>, 5> {
    // WGSL matrices are column-major: m[col][row].
    // Row i = (m[0][i], m[1][i], m[2][i], m[3][i]).
    let row0 = vec4<f32>(m[0][0], m[1][0], m[2][0], m[3][0]);
    let row1 = vec4<f32>(m[0][1], m[1][1], m[2][1], m[3][1]);
    let row2 = vec4<f32>(m[0][2], m[1][2], m[2][2], m[3][2]);
    let row3 = vec4<f32>(m[0][3], m[1][3], m[2][3], m[3][3]);

    return array<vec4<f32>, 5>(
        row3 + row0, // Left:   clip x >= -w
        row3 - row0, // Right:  clip x <=  w
        row3 + row1, // Bottom: clip y >= -w
        row3 - row1, // Top:    clip y <=  w
        row2,        // Near:   clip z >=  0  (WGPU depth range [0, 1])
    );
}

// Returns true if the AABB is fully outside the half-space defined by the plane.
fn outside_plane(plane: vec4<f32>, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> bool {
    // The "positive vertex" is the AABB corner most along the plane normal.
    // If it is still on the negative side, the whole AABB is outside.
    let p = vec3<f32>(
        select(aabb_min.x, aabb_max.x, plane.x >= 0.0),
        select(aabb_min.y, aabb_max.y, plane.y >= 0.0),
        select(aabb_min.z, aabb_max.z, plane.z >= 0.0),
    );
    return dot(plane.xyz, p) + plane.w < 0.0;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let chunk_index = id.x;
    if chunk_index >= arrayLength(&chunks) {
        return;
    }

    let chunk = chunks[chunk_index];
    let world_min = vec3<f32>(chunk.position) * CHUNK_SIZE;
    let world_max = world_min + CHUNK_SIZE;

    let planes = extract_planes(view_proj);
    for (var i = 0; i < 5; i++) {
        if outside_plane(planes[i], world_min, world_max) {
            return;
        }
    }

    let slot = atomicAdd(&visible_chunks.count, 1u);
    visible_chunks.indices[slot] = chunk_index;
}
