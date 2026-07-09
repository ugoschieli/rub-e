struct Camera {
    view_matrix: mat4x4<f32>,
    inv_view_matrix: mat4x4<f32>,
    world_pos: vec4<f32>,
}

struct Ray {
    orig: vec3<f32>,
    dir: vec3<f32>,
}

struct Grid {
    // Number of voxels along each axis.
    size: vec3<u32>,
    // Edge length of one (cubic) voxel in world units, i.e. meters.
    voxel_size: f32,
}

struct Entry {
    t: f32,
    hit: bool,
}

// Running state of a Digital Differential Analyzer walking a voxel grid.
struct Dda {
    // Coordinates of the voxel currently under the ray.
    voxel: vec3<i32>,
    // Stepping direction on each axis (+1 or -1).
    step: vec3<i32>,
    // Ray parameter t at which the next voxel boundary is crossed, per axis.
    t_max: vec3<f32>,
    // Increment in t needed to cross one full voxel, per axis.
    t_delta: vec3<f32>,
    // Ray parameter t at which the current voxel was entered.
    t: f32,
    // Outward normal of the face the current voxel was entered through.
    normal: vec3<f32>,
}

struct Hit {
    hit: bool,
    voxel: vec3<i32>,
    normal: vec3<f32>,
    t: f32,
}

@group(0) @binding(0) var out_tex: texture_storage_2d<bgra8unorm, write>;
@group(0) @binding(1) var<uniform> cam: Camera;
// One bit per voxel, packed into u32 words in x-major order
// (index = x + y*size.x + z*size.x*size.y). Generated on the CPU from noise.
@group(0) @binding(2) var<storage, read> occupancy: array<u32>;
// Grid dimensions and voxel size, owned by the CPU so the two never drift.
@group(0) @binding(3) var<uniform> grid: Grid;

fn primary_ray(pixel: vec2<u32>, cam: Camera, resolution: vec2<u32>) -> Ray {
    let uv = (vec2<f32>(pixel) + vec2<f32>(0.5)) / vec2<f32>(resolution);
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    // Reconstruct a point on the NEAR plane (NDC z = 0). The far plane (z = 1)
    // is a singularity under an infinite-far projection: it maps to a point at
    // infinity, so the w-divide yields NaN and every ray reports a miss.
    var near = cam.inv_view_matrix * vec4<f32>(ndc, 0.0, 1.0);
    near /= near.w;

    return Ray(cam.world_pos.xyz, normalize(near.xyz - cam.world_pos.xyz));
}

fn grid_entry(ray: Ray, grid: Grid) -> Entry {
    // Use the classic ray-slab intersection method to determine the ray entry into the voxel grid
    let inv_dir = 1 / ray.dir;
    let t_low = (vec3<f32>(0) - ray.orig) * inv_dir;
    let t_high = (vec3<f32>(grid.size) - ray.orig) * inv_dir;

    let t_close = min(t_low, t_high);
    let t_far = max(t_low, t_high);

    let t_enter = max(max(t_close.x, t_close.y), t_close.z);
    let t_exit = min(min(t_far.x, t_far.y), t_far.z);

    var entry = Entry();
    // t_enter <= t_exit: the line crosses the box.
    // t_exit >= 0: the crossing is in front of the ray, not behind it.
    entry.hit = t_enter <= t_exit && t_exit >= 0.0;
    entry.t = t_enter;

    return entry;
}

// Set up DDA state for a ray that enters the grid at parameter `t_enter`.
fn dda_init(ray: Ray, grid: Grid, t_enter: f32) -> Dda {
    let inv_dir = 1.0 / ray.dir;
    let positive = ray.dir >= vec3<f32>(0.0);

    // Clamp the entry point into the grid so the starting voxel is always valid,
    // even when the ray grazes a boundary and floating point nudges it outside.
    let p = ray.orig + t_enter * ray.dir;
    let voxel = vec3<i32>(clamp(floor(p), vec3<f32>(0.0), vec3<f32>(grid.size) - 1.0));

    let step = select(vec3<i32>(-1), vec3<i32>(1), positive);

    // Distance in t to cross a full unit voxel along each axis.
    let t_delta = abs(inv_dir);

    // t at the next boundary: voxel+1 when stepping forward, voxel when stepping
    // backward. Expressed relative to the ray origin so it stays absolute in t.
    let next = vec3<f32>(voxel) + select(vec3<f32>(0.0), vec3<f32>(1.0), positive);
    let t_max = (next - ray.orig) * inv_dir;

    // The grid face we came in through is the slab axis whose near-plane
    // crossing produced t_enter; its outward normal opposes the step direction.
    let t_close = min((vec3<f32>(0.0) - ray.orig) * inv_dir, (vec3<f32>(grid.size) - ray.orig) * inv_dir);
    let entry_axis = t_close >= vec3<f32>(t_enter);
    let normal = select(vec3<f32>(0.0), -vec3<f32>(step), entry_axis);

    return Dda(voxel, step, t_max, t_delta, t_enter, normal);
}

// Advance the DDA by one voxel along the axis whose boundary is nearest.
fn dda_step(dda: Dda) -> Dda {
    var d = dda;

    // Axis (or axes, on an exact corner crossing) with the smallest t_max.
    let mask = vec3<bool>(
        d.t_max.x <= d.t_max.y && d.t_max.x <= d.t_max.z,
        d.t_max.y <= d.t_max.x && d.t_max.y <= d.t_max.z,
        d.t_max.z <= d.t_max.x && d.t_max.z <= d.t_max.y,
    );

    d.t = min(min(d.t_max.x, d.t_max.y), d.t_max.z);
    d.voxel += select(vec3<i32>(0), d.step, mask);
    d.normal = select(vec3<f32>(0.0), -vec3<f32>(d.step), mask);
    d.t_max += select(vec3<f32>(0.0), d.t_delta, mask);

    return d;
}

// Occupancy test for an in-bounds voxel: unpack its bit from the storage buffer.
fn is_solid(voxel: vec3<i32>, grid: Grid) -> bool {
    let size = vec3<i32>(grid.size);
    let index = voxel.x + voxel.y * size.x + voxel.z * size.x * size.y;
    return (occupancy[index / 32] & (1u << u32(index % 32))) != 0u;
}

// Walk the grid voxel by voxel until a solid voxel is hit or the ray leaves.
fn dda_traverse(ray: Ray, grid: Grid) -> Hit {
    var hit = Hit();

    // Traverse in grid space, where each voxel is a unit cube. Scaling the world
    // ray's origin and direction by 1/voxel_size preserves the ray parameter t,
    // so distances (hit.t) come back out directly in world units (meters).
    let inv_size = 1.0 / grid.voxel_size;
    let gray = Ray(ray.orig * inv_size, ray.dir * inv_size);

    let entry = grid_entry(gray, grid);
    if !entry.hit {
        return hit;
    }

    var dda = dda_init(gray, grid, max(entry.t, 0.0));

    // Each axis boundary is crossed at most `size` times, so the total number
    // of steps is bounded by the sum of the grid dimensions.
    let max_steps = grid.size.x + grid.size.y + grid.size.z;
    for (var i = 0u; i < max_steps; i += 1u) {
        if any(dda.voxel < vec3<i32>(0)) || any(dda.voxel >= vec3<i32>(grid.size)) {
            break;
        }
        if is_solid(dda.voxel, grid) {
            hit.hit = true;
            hit.voxel = dda.voxel;
            hit.normal = dda.normal;
            hit.t = dda.t;
            return hit;
        }
        dda = dda_step(dda);
    }

    return hit;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dimensions = textureDimensions(out_tex);
    if gid.x >= dimensions.x || gid.y >= dimensions.y {
        return;
    }

    let ray = primary_ray(gid.xy, cam, dimensions);
    let hit = dda_traverse(ray, grid);
    if !hit.hit {
        textureStore(out_tex, vec2<i32>(gid.xy), vec4<f32>(0.0, 0.0, 0.0, 1.0));
        return;
    }

    let light = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let diffuse = max(dot(hit.normal, light), 0.0) * 0.8 + 0.2;
    let tint = vec3<f32>(0.4);

    var color = tint * diffuse;
    color = pow(color, vec3<f32>(1.0 / 2.2));
    textureStore(out_tex, vec2<i32>(gid.xy), vec4<f32>(color, 1.0));
}
