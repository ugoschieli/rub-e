// Per-voxel ray-box intersection renderer.
//
// Each voxel is drawn as one instanced oriented box (a coarse proxy). The
// fragment shader reconstructs the camera ray and runs the branchless ray-box
// test from Majercik, Crassin, Shirley & McGuire, "A Ray-Box Intersection
// Algorithm and Efficient Dynamic Voxel Rendering" (JCGT 2018) to recover the
// exact front-face hit point and surface normal, then shades and writes depth.
//
// Boxes are arbitrarily oriented and sized: the ray is transformed into each
// box's local frame (Listing 5, `oriented` branch), tested against per-axis
// half-extents, and the resulting normal is rotated back to world space.

// Full per-box data, produced by an updater. The hot fields (rotation, center)
// change every frame; the cold fields (half_extent, color) are seeded once.
struct BoxTransform {
    rotation: vec4<f32>,     // object-to-world orientation, quaternion (x, y, z, w)
    center: vec3<f32>,
    half_extent: f32,        // uniform scale on every axis, so boxes stay cubic
    color: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) center: vec3<f32>,
    @location(2) half_extents: vec3<f32>,
    @location(3) @interpolate(flat) rotation: vec4<f32>,
    @location(4) @interpolate(flat) color: u32,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

struct BoxHit {
    hit: bool,
    dist: f32,
    normal: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: mat4x4<f32>;
@group(0) @binding(1) var<uniform> camera_pos: vec4<f32>;
@group(0) @binding(2) var<storage, read> transforms: array<BoxTransform>;
// Compacted indices of the boxes that survived GPU frustum culling, written by
// `box_cull.wgsl`. The indirect draw issues one instance per survivor, so the
// instance index selects a slot here, which in turn selects the real box.
@group(0) @binding(3) var<storage, read> visible_indices: array<u32>;

// Unit cube corners in [0,1]^3, 12 triangles (36 vertices), outward faces wound
// CCW to match FrontFace::Ccw. The pipeline culls front faces, so the back faces
// are rasterized — this guarantees a covering fragment even when the camera sits
// inside the box (handled by the can_start_in_box winding flip below). A proper
// rotation preserves winding, so this stays valid for oriented boxes.
const BOX_VERTICES = array<vec3<f32>, 36>(
    // -Z face
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 0.0),
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0),
    // +Z face
    vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(1.0, 0.0, 1.0), vec3<f32>(1.0, 1.0, 1.0),
    vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(0.0, 1.0, 1.0),
    // -X face
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 1.0, 1.0),
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 1.0), vec3<f32>(0.0, 1.0, 0.0),
    // +X face
    vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 1.0),
    vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(1.0, 0.0, 1.0),
    // -Y face
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 1.0),
    vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 0.0, 1.0), vec3<f32>(0.0, 0.0, 1.0),
    // +Y face
    vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 1.0, 1.0), vec3<f32>(1.0, 1.0, 1.0),
    vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 1.0, 1.0), vec3<f32>(1.0, 1.0, 0.0),
);

// Rotate vector `v` by unit quaternion `q` (object -> world).
fn quat_rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let u = q.xyz;
    return v + 2.0 * cross(u, cross(u, v) + q.w * v);
}

// Rotate vector `v` by the inverse (conjugate) of unit quaternion `q` (world -> object).
fn quat_rotate_inv(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    return quat_rotate(vec4<f32>(-q.xyz, q.w), v);
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let index = visible_indices[instance_index];
    let t = transforms[index];
    // [0,1] corner -> [-half, +half] local -> rotate -> translate to center.
    // Uniform half-extent: the scalar broadcasts to all three axes, so the proxy
    // stays a cube.
    let half_extents = vec3<f32>(t.half_extent);
    let local = (BOX_VERTICES[vertex_index] * 2.0 - 1.0) * half_extents;
    let world_pos = t.center + quat_rotate(t.rotation, local);

    out.clip_position = camera * vec4<f32>(world_pos, 1.0);
    out.world_pos = world_pos;
    out.center = t.center;
    out.half_extents = half_extents;
    out.rotation = t.rotation;
    out.color = t.color;

    return out;
}

// Oriented ray-box intersection. The ray is moved into the box's local frame
// (world -> object via the inverse rotation), tested against the axis-aligned
// `radius` (half-extents) there, and the chosen face normal is rotated back to
// world space. Returns whether the ray hits, the hit distance along world `rd`,
// and the world-space normal.
fn ray_box_intersect(
    ro: vec3<f32>,
    rd: vec3<f32>,
    center: vec3<f32>,
    radius: vec3<f32>,
    rotation: vec4<f32>,
    can_start_in_box: bool,
) -> BoxHit {
    // Into box-local space. Rotation is orthonormal, so the hit parameter t
    // (distance along rd) is preserved and usable directly in world space.
    let o = quat_rotate_inv(rotation, ro - center);
    let d = quat_rotate_inv(rotation, rd);
    let inv_d = 1.0 / d;

    var winding = 1.0;
    if can_start_in_box &&
       max(abs(o.x) / radius.x, max(abs(o.y) / radius.y, abs(o.z) / radius.z)) < 1.0 {
        winding = -1.0;
    }

    let sgn = -sign(d);
    let dist3 = (radius * winding * sgn - o) * inv_d;

    let tx = (dist3.x >= 0.0) && (abs(o.y + d.y * dist3.x) < radius.y) && (abs(o.z + d.z * dist3.x) < radius.z);
    let ty = (dist3.y >= 0.0) && (abs(o.x + d.x * dist3.y) < radius.x) && (abs(o.z + d.z * dist3.y) < radius.z);
    let tz = (dist3.z >= 0.0) && (abs(o.x + d.x * dist3.z) < radius.x) && (abs(o.y + d.y * dist3.z) < radius.y);

    var n = vec3<f32>(0.0);
    var dist = 0.0;
    if tx {
        n = vec3<f32>(sgn.x, 0.0, 0.0);
        dist = dist3.x;
    } else if ty {
        n = vec3<f32>(0.0, sgn.y, 0.0);
        dist = dist3.y;
    } else if tz {
        n = vec3<f32>(0.0, 0.0, sgn.z);
        dist = dist3.z;
    }

    var out: BoxHit;
    out.hit = tx || ty || tz;
    out.dist = dist;
    out.normal = quat_rotate(rotation, n); // back to world space
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    let ro = camera_pos.xyz;
    let rd = in.world_pos - ro;

    let hit = ray_box_intersect(ro, rd, in.center, in.half_extents, in.rotation, true);
    if !hit.hit {
        discard;
    }

    let hit_world = ro + rd * hit.dist;
    let clip = camera * vec4<f32>(hit_world, 1.0);

    let base = vec3<f32>(
        f32(in.color & 1023u) / 1023.0,
        f32((in.color >> 10u) & 1023u) / 1023.0,
        f32((in.color >> 20u) & 1023u) / 1023.0,
    );

    // Simple Lambert shading from a fixed light so the oriented faces read distinctly.
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let diffuse = max(dot(hit.normal, light_dir), 0.0);
    let shade = 0.25 + 0.75 * diffuse;

    // Swap for `hit.normal * 0.5 + 0.5` to debug normals directly.
    var out: FragmentOutput;
    out.color = vec4<f32>(base * shade, 1.0);
    out.depth = clip.z / clip.w;
    return out;
}