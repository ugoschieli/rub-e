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
// Coarse occupancy: one bit per BRICK^3 block, set if the block holds any solid
// voxel. Lets the traversal skip empty space a whole brick at a time.
@group(0) @binding(4) var<storage, read> brick_occupancy: array<u32>;

// Edge length of a brick, in voxels.
const BRICK: u32 = 8u;
// Sentinel "no distance limit", for unbounded primary and shadow rays.
const T_MAX: f32 = 1e30;

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

// Number of bricks along each axis, ceil(size / BRICK).
fn brick_dims(grid: Grid) -> vec3<i32> {
    return vec3<i32>((grid.size + (BRICK - 1u)) / BRICK);
}

// True if a brick holds at least one solid voxel (its coarse occupancy bit).
fn brick_solid(brick: vec3<i32>, grid: Grid) -> bool {
    let dims = brick_dims(grid);
    let index = brick.x + brick.y * dims.x + brick.z * dims.x * dims.y;
    return (brick_occupancy[index / 32] & (1u << u32(index % 32))) != 0u;
}

// Fine per-voxel DDA restricted to one (non-empty) brick. Returns the first
// solid voxel hit before the ray leaves the brick or passes `max_t`. `entry_t`
// and `entry_normal` describe the face the ray crossed to enter the brick, so a
// solid voxel sitting on that face reports the correct normal.
fn trace_brick(gray: Ray, grid: Grid, brick: vec3<i32>, entry_t: f32, entry_normal: vec3<f32>, max_t: f32) -> Hit {
    var hit = Hit();

    let inv_dir = 1.0 / gray.dir;
    let positive = gray.dir >= vec3<f32>(0.0);
    let step = select(vec3<i32>(-1), vec3<i32>(1), positive);

    // Voxel range this brick covers (clamped to the grid for boundary bricks).
    // Clamping the entry voxel into it stops floating point on the brick face
    // from starting us in a neighbour.
    let vmin = brick * i32(BRICK);
    let vmax = min(vmin + (i32(BRICK) - 1), vec3<i32>(grid.size) - 1);
    let voxel = clamp(vec3<i32>(floor(gray.orig + entry_t * gray.dir)), vmin, vmax);

    let next = vec3<f32>(voxel) + select(vec3<f32>(0.0), vec3<f32>(1.0), positive);
    var dda = Dda(voxel, step, (next - gray.orig) * inv_dir, abs(inv_dir), entry_t, entry_normal);

    // A ray crosses at most BRICK cells per axis inside a single brick.
    for (var i = 0u; i < 3u * BRICK; i += 1u) {
        if any(dda.voxel < vmin) || any(dda.voxel > vmax) {
            break; // left the brick
        }
        if dda.t > max_t {
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

// Hierarchical DDA: walk the coarse brick grid and only descend into a fine
// per-voxel DDA for bricks that actually contain geometry, so empty space costs
// one step per brick instead of one per voxel. `max_t` bounds the ray in world
// units (pass T_MAX for none). Distances are returned in world units.
fn trace(ray: Ray, grid: Grid, max_t: f32) -> Hit {
    var hit = Hit();

    // Grid space (unit voxels); scaling origin and dir together preserves t.
    let inv_size = 1.0 / grid.voxel_size;
    let gray = Ray(ray.orig * inv_size, ray.dir * inv_size);

    let entry = grid_entry(gray, grid);
    if !entry.hit {
        return hit;
    }

    let inv_dir = 1.0 / gray.dir;
    let positive = gray.dir >= vec3<f32>(0.0);
    let step = select(vec3<i32>(-1), vec3<i32>(1), positive);
    let dims = brick_dims(grid);
    let t_start = max(entry.t, 0.0);

    // Starting brick and its coarse DDA state. A brick is BRICK voxels wide, so
    // the coarse t_delta is the fine one scaled by BRICK; we reuse `dda_step`.
    let p = gray.orig + t_start * gray.dir;
    let brick = clamp(vec3<i32>(floor(p / f32(BRICK))), vec3<i32>(0), dims - vec3<i32>(1));
    let next = (vec3<f32>(brick) + select(vec3<f32>(0.0), vec3<f32>(1.0), positive)) * f32(BRICK);

    // Face the ray entered the grid through (for the first brick's fine trace).
    let t_close = min((vec3<f32>(0.0) - gray.orig) * inv_dir, (vec3<f32>(grid.size) - gray.orig) * inv_dir);
    let entry_normal = select(vec3<f32>(0.0), -vec3<f32>(step), t_close >= vec3<f32>(entry.t));

    var coarse = Dda(brick, step, (next - gray.orig) * inv_dir, abs(inv_dir) * f32(BRICK), t_start, entry_normal);

    let max_bricks = dims.x + dims.y + dims.z;
    for (var i = 0; i < max_bricks; i += 1) {
        if coarse.t > max_t {
            break;
        }
        if any(coarse.voxel < vec3<i32>(0)) || any(coarse.voxel >= dims) {
            break;
        }
        if brick_solid(coarse.voxel, grid) {
            let h = trace_brick(gray, grid, coarse.voxel, coarse.t, coarse.normal, max_t);
            if h.hit {
                return h;
            }
        }
        coarse = dda_step(coarse);
    }

    return hit;
}

// Direction from any surface toward the (infinitely far) directional light.
// z is up, so the light sits high in the sky with a slight horizontal slant.
const LIGHT_DIR = vec3<f32>(-0.35, 0.7, 0.5);
// Offset the shadow-ray origin this fraction of a voxel off the surface along
// the normal so it starts in the empty neighbour and never self-shadows.
const SHADOW_BIAS: f32 = 0.01;
// Angular radius of the light disc (tan of the half-angle). Larger -> softer
// penumbra; 0 collapses back to a hard shadow.
const LIGHT_RADIUS: f32 = 0.03;
// Shadow rays averaged per pixel. More -> less noise, linearly more cost.
const SHADOW_SAMPLES: u32 = 4u;
// How far, in voxels, shadow rays search for an occluder. Bounds the cost of
// rays that skim the terrain; occluders past this cast no shadow. Set very large
// (e.g. 1e9) to disable and get fully global shadows again.
const SHADOW_MAX_VOXELS: f32 = 48.0;
// Reach of ambient occlusion, in voxels; only occluders within this range darken.
const AO_RADIUS_VOXELS: f32 = 4.0;
// Hemisphere rays averaged per pixel for AO.
const AO_SAMPLES: u32 = 4u;

// PCG-style hash used to seed the per-pixel RNG.
fn hash_u32(x: u32) -> u32 {
    var v = x * 747796405u + 2891336453u;
    v = ((v >> ((v >> 28u) + 4u)) ^ v) * 277803737u;
    return (v >> 22u) ^ v;
}

// Advance the RNG and return a float in [0, 1).
fn rand01(state: ptr<function, u32>) -> f32 {
    *state = *state * 747796405u + 2891336453u;
    let v = ((*state >> ((*state >> 28u) + 4u)) ^ *state) * 277803737u;
    return f32((v >> 22u) ^ v) / 4294967296.0;
}

// Fraction of the light disc visible from the surface point, in [0, 1]. Samples
// several shadow rays jittered across the light's angular size and averages
// their visibility, producing a soft penumbra that widens with occluder
// distance (the sample cone diverges).
fn soft_shadow(hit_pos: vec3<f32>, normal: vec3<f32>, grid: Grid, seed: u32) -> f32 {
    let light = normalize(LIGHT_DIR);
    // Orthonormal basis spanning the plane perpendicular to the light.
    let up = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(light.y) > 0.99);
    let tangent = normalize(cross(up, light));
    let bitangent = cross(light, tangent);

    let origin = hit_pos + normal * grid.voxel_size * SHADOW_BIAS;
    let max_t = SHADOW_MAX_VOXELS * grid.voxel_size;

    var state = seed;
    var visible = 0u;
    for (var i = 0u; i < SHADOW_SAMPLES; i += 1u) {
        // Uniform sample of the light disc, offsetting the shadow direction.
        let angle = 6.2831853 * rand01(&state);
        let radius = sqrt(rand01(&state)) * LIGHT_RADIUS;
        let offset = (tangent * cos(angle) + bitangent * sin(angle)) * radius;
        let dir = normalize(light + offset);

        if !trace(Ray(origin, dir), grid, max_t).hit {
            visible += 1u;
        }
    }

    return f32(visible) / f32(SHADOW_SAMPLES);
}

// Fraction of the hemisphere above the surface that is open, in [0, 1]. Casts
// cosine-weighted rays around the normal and counts those that reach `max_t`
// without hitting anything; nearby geometry drives the value toward 0.
fn ambient_occlusion(hit_pos: vec3<f32>, normal: vec3<f32>, grid: Grid, seed: u32) -> f32 {
    // Orthonormal basis around the surface normal.
    let up = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(normal.y) > 0.99);
    let tangent = normalize(cross(up, normal));
    let bitangent = cross(normal, tangent);

    let origin = hit_pos + normal * grid.voxel_size * SHADOW_BIAS;
    let max_t = AO_RADIUS_VOXELS * grid.voxel_size;

    var state = seed;
    var open = 0u;
    for (var i = 0u; i < AO_SAMPLES; i += 1u) {
        // Cosine-weighted hemisphere sample in local space (z = normal).
        let r1 = rand01(&state);
        let r2 = rand01(&state);
        let r = sqrt(r1);
        let phi = 6.2831853 * r2;
        let local = vec3<f32>(r * cos(phi), r * sin(phi), sqrt(1.0 - r1));
        let dir = tangent * local.x + bitangent * local.y + normal * local.z;

        if !trace(Ray(origin, dir), grid, max_t).hit {
            open += 1u;
        }
    }

    return f32(open) / f32(AO_SAMPLES);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dimensions = textureDimensions(out_tex);
    if gid.x >= dimensions.x || gid.y >= dimensions.y {
        return;
    }

    let ray = primary_ray(gid.xy, cam, dimensions);
    let hit = trace(ray, grid, T_MAX);
    if !hit.hit {
        textureStore(out_tex, vec2<i32>(gid.xy), vec4<f32>(0.0, 0.0, 0.0, 1.0));
        return;
    }

    let hit_pos = ray.orig + hit.t * ray.dir;
    let seed = hash_u32(gid.x + gid.y * dimensions.x);

    // Direct light: Lambert, attenuated by the fraction of the light disc visible.
    let light = normalize(LIGHT_DIR);
    var shade = max(dot(hit.normal, light), 0.0);
    if shade > 0.0 {
        shade *= soft_shadow(hit_pos, hit.normal, grid, seed);
    }

    // Ambient term darkened by how enclosed the surface is.
    let ao = ambient_occlusion(hit_pos, hit.normal, grid, seed ^ 0x9e3779b9u);

    let diffuse = shade * 0.8 + 0.2 * ao;
    let tint = vec3<f32>(0.4);

    var color = tint * diffuse;
    color = pow(color, vec3<f32>(1.0 / 2.2));
    textureStore(out_tex, vec2<i32>(gid.xy), vec4<f32>(color, 1.0));
}
