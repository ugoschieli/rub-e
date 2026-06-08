// GPU frustum-culling pass for the dynamic oriented-box renderer.
//
// One invocation per box. Each oriented box is reduced to a conservative
// world-space AABB and tested against the 5 frustum planes (Gribb-Hartmann; the
// far plane is omitted because the camera uses an infinite projection). The
// indices of the survivors are appended to `visible_indices`, and the survivor
// count is written straight into the indirect draw's `instance_count`, so the
// render pass draws only the boxes that pass the test.
//
// Boxes live in raw world space and `view_proj` already folds in the base
// change (see `Camera::matrix`), so the extracted planes are expressed in the
// same raw world space as `box.center` — no extra transform is needed here.

// Full per-box data, produced by an updater. Only `center`/`rotation` and the
// uniform `half_extent` matter to culling; `color` is ignored here.
struct BoxTransform {
    rotation: vec4<f32>,     // object-to-world orientation, quaternion (x, y, z, w)
    center: vec3<f32>,
    half_extent: f32,        // uniform scale on every axis, so boxes stay cubic
    color: u32,
}

// Mirrors `wgpu::util::DrawIndirectArgs`: vertex_count, instance_count,
// first_vertex, first_instance. `instance_count` is the atomic survivor counter,
// reset to 0 on the CPU before this pass runs.
struct DrawArgs {
    vertex_count: u32,
    instance_count: atomic<u32>,
    first_vertex: u32,
    first_instance: u32,
}

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(1) var<storage, read> transforms: array<BoxTransform>;
@group(0) @binding(2) var<storage, read_write> draw_args: DrawArgs;
@group(0) @binding(3) var<storage, read_write> visible_indices: array<u32>;

// Rotate vector `v` by unit quaternion `q` (object -> world).
fn quat_rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let u = q.xyz;
    return v + 2.0 * cross(u, cross(u, v) + q.w * v);
}

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
    let index = id.x;
    if index >= arrayLength(&transforms) {
        return;
    }

    let t = transforms[index];
    let he = t.half_extent;

    // Conservative world AABB of the oriented cube. Rotating each half-extent
    // basis vector into world space and summing their absolute components gives
    // the AABB half-size (the box's silhouette along each world axis).
    let ex = abs(quat_rotate(t.rotation, vec3<f32>(he, 0.0, 0.0)));
    let ey = abs(quat_rotate(t.rotation, vec3<f32>(0.0, he, 0.0)));
    let ez = abs(quat_rotate(t.rotation, vec3<f32>(0.0, 0.0, he)));
    let half_size = ex + ey + ez;
    let world_min = t.center - half_size;
    let world_max = t.center + half_size;

    let planes = extract_planes(view_proj);
    for (var i = 0; i < 5; i++) {
        if outside_plane(planes[i], world_min, world_max) {
            return;
        }
    }

    // Survivor: claim a slot in the compacted list. The slot is also the running
    // instance count consumed by the indirect draw.
    let slot = atomicAdd(&draw_args.instance_count, 1u);
    visible_indices[slot] = index;
}