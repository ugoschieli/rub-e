// GPU motion pass for the dynamic oriented-box renderer.
//
// Each box flies along its own fixed circular orbit. The orbit (center, radius,
// in-plane basis, angular speed, phase) is static per box and uploaded once; this
// pass recomputes the box's world-space position and orientation from a single
// `time` uniform every frame, so per-frame CPU work and upload are constant-size
// regardless of box count. Runs before the cull pass, which reads the result.

// Full per-box data. This pass overwrites only the hot fields (rotation, center);
// the cold fields (half_extent, color) are seeded once and left in place, so the
// struct must still match the 48-byte layout for the field offsets to be correct.
struct BoxTransform {
    rotation: vec4<f32>,     // object-to-world orientation, quaternion (x, y, z, w)
    center: vec3<f32>,
    half_extent: f32,
    color: u32,
}

// Cold, static orbit definition: world-space ellipse-circle in the plane spanned
// by the orthonormal basis (u, v) around `center`. `pos(t) = center + radius *
// (cos(theta) * u + sin(theta) * v)` with `theta = phase + speed * t`.
struct Orbit {
    center: vec3<f32>,
    radius: f32,
    u: vec3<f32>,
    speed: f32,
    v: vec3<f32>,
    phase: f32,
}

// `time` plus this pass's `[base, base + count)` slice of the global transform
// array. `orbits` is indexed locally; transforms are written at `base + local`.
struct Params {
    time: f32,
    base: u32,
    count: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> orbits: array<Orbit>;
@group(0) @binding(2) var<storage, read_write> transforms: array<BoxTransform>;

// Unit quaternion for a rotation of `angle` about unit `axis`.
fn quat_axis_angle(axis: vec3<f32>, angle: f32) -> vec4<f32> {
    let h = angle * 0.5;
    return vec4<f32>(axis * sin(h), cos(h));
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if local >= params.count {
        return;
    }

    let o = orbits[local];
    let theta = o.phase + o.speed * params.time;

    // Position on the orbit; orientation tumbles about the orbit's plane normal.
    let pos = o.center + o.radius * (cos(theta) * o.u + sin(theta) * o.v);
    let normal = normalize(cross(o.u, o.v));

    transforms[params.base + local].center = pos;
    transforms[params.base + local].rotation = quat_axis_angle(normal, theta);
}
