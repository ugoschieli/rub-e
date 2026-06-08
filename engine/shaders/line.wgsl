// GPU motion pass: each box oscillates back and forth along a fixed straight
// line. The line (anchor, direction, amplitude, speed, phase) is static per box
// and uploaded once; this pass recomputes the box's world-space position and
// orientation from a single `time` uniform every frame. Mirrors the structure
// of `orbit.wgsl`; runs before the cull pass, which reads the result.

// Full per-box data. This pass overwrites only the hot fields (rotation, center);
// the cold fields (half_extent, color) are seeded once and left in place, so the
// struct must still match the 48-byte layout for the field offsets to be correct.
struct BoxTransform {
    rotation: vec4<f32>,     // object-to-world orientation, quaternion (x, y, z, w)
    center: vec3<f32>,
    half_extent: f32,
    color: u32,
}

// Cold, static line definition. `pos(t) = anchor + dir * amplitude *
// sin(phase + speed * t)`; the box also spins about `axis`.
struct Line {
    anchor: vec3<f32>,   // line midpoint in raw world space
    amplitude: f32,      // half-length of the travel
    dir: vec3<f32>,      // unit travel direction
    speed: f32,          // oscillation angular speed, rad/s
    axis: vec3<f32>,     // unit spin axis
    phase: f32,          // initial angle
}

// `time` plus this pass's `[base, base + count)` slice of the global transform
// array. `lines` is indexed locally; transforms are written at `base + local`.
struct Params {
    time: f32,
    base: u32,
    count: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> lines: array<Line>;
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

    let l = lines[local];
    let phi = l.phase + l.speed * params.time;

    let pos = l.anchor + l.dir * (l.amplitude * sin(phi));

    transforms[params.base + local].center = pos;
    transforms[params.base + local].rotation = quat_axis_angle(l.axis, phi);
}
