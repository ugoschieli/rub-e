struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    prev_view_proj: mat4x4<f32>,
};

struct Plane {
    normal: vec3<f32>,
    distance: f32,
}

struct Frustum {
    planes: array<Plane, 6>,
}

struct AABB {
    min: vec3<f32>,
    max: vec3<f32>,
}

struct CubeRaw {
    model: mat4x4<f32>,
    color: vec4<f32>,
}

struct DrawIndexedIndirectArgs {
    index_count: u32,
    instance_count: atomic<u32>,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<storage, read> all_instances: array<CubeRaw>;

@group(1) @binding(1)
var<storage, read_write> visible_instances: array<CubeRaw>;

@group(1) @binding(2)
var<storage, read_write> draw_cmd: DrawIndexedIndirectArgs;

// Occlusion Texture (Hi-Z)
@group(1) @binding(3)
var hiz_texture: texture_2d<f32>;

fn normalize_plane(v: vec4<f32>) -> Plane {
    let len = length(v.xyz);
    return Plane(v.xyz / len, v.w / len);
}

fn frustum_from_view_proj(view_proj: mat4x4<f32>) -> Frustum {
    // Transpose to get rows as vectors
    let m = transpose(view_proj);
    let row0 = m[0];
    let row1 = m[1];
    let row2 = m[2];
    let row3 = m[3];

    var planes: array<Plane, 6>;
    planes[0] = normalize_plane(row3 + row0); // Left
    planes[1] = normalize_plane(row3 - row0); // Right
    planes[2] = normalize_plane(row3 + row1); // Bottom
    planes[3] = normalize_plane(row3 - row1); // Top
    planes[4] = normalize_plane(row3 + row2); // Near
    planes[5] = normalize_plane(row3 - row2); // Far
    return Frustum(planes);
}

fn plane_distance(plane: Plane, point: vec3<f32>) -> f32 {
    return dot(plane.normal, point) + plane.distance;
}

fn intersects_aabb(frustum: Frustum, aabb: AABB) -> bool {
    for (var i = 0; i < 6; i++) {
        let plane = frustum.planes[i];
        let px = select(aabb.min.x, aabb.max.x, plane.normal.x >= 0.0);
        let py = select(aabb.min.y, aabb.max.y, plane.normal.y >= 0.0);
        let pz = select(aabb.min.z, aabb.max.z, plane.normal.z >= 0.0);
        if (plane_distance(plane, vec3<f32>(px, py, pz)) < 0.0) {
            return false;
        }
    }
    return true;
}

fn get_aabb_from_instance(model: mat4x4<f32>) -> AABB {
    let world_center = model[3].xyz;
    let right = model[0].xyz;
    let up = model[1].xyz;
    let forward = model[2].xyz;
    let new_extent_x = (abs(right.x) + abs(up.x) + abs(forward.x)) * 0.5;
    let new_extent_y = (abs(right.y) + abs(up.y) + abs(forward.y)) * 0.5;
    let new_extent_z = (abs(right.z) + abs(up.z) + abs(forward.z)) * 0.5;
    let new_extent = vec3<f32>(new_extent_x, new_extent_y, new_extent_z);
    return AABB(world_center - new_extent, world_center + new_extent);
}

// Check if AABB is occluded by the Hi-Z buffer
// Returns true if OCCLUDED (hidden)
fn is_occluded(aabb: AABB) -> bool {
    // Project AABB corners to clip space
    // We only need min/max in screen space.
    // Conservative approach: project all 8 corners.
    
    var min_p = vec3<f32>(1.0, 1.0, 1.0);
    var max_p = vec3<f32>(-1.0, -1.0, -1.0);

    let corners = array<vec3<f32>, 8>(
        vec3<f32>(aabb.min.x, aabb.min.y, aabb.min.z),
        vec3<f32>(aabb.max.x, aabb.min.y, aabb.min.z),
        vec3<f32>(aabb.min.x, aabb.max.y, aabb.min.z),
        vec3<f32>(aabb.max.x, aabb.max.y, aabb.min.z),
        vec3<f32>(aabb.min.x, aabb.min.y, aabb.max.z),
        vec3<f32>(aabb.max.x, aabb.min.y, aabb.max.z),
        vec3<f32>(aabb.min.x, aabb.max.y, aabb.max.z),
        vec3<f32>(aabb.max.x, aabb.max.y, aabb.max.z)
    );

    for (var i = 0; i < 8; i++) {
        // Use PREVIOUS view-projection matrix for Hi-Z check
        let clip_pos = camera.prev_view_proj * vec4<f32>(corners[i], 1.0);
        // If w <= 0, point is behind camera. This makes screen space projection tricky.
        // But we already passed Frustum culling, so at least partially visible?
        // Actually, for occlusion, if it intersects near plane, we should assume visible.
        if (clip_pos.w <= 0.0) {
            return false; // Intersection with near plane, treat as visible
        }

        let ndc = clip_pos.xyz / clip_pos.w;
        min_p = min(min_p, ndc);
        max_p = max(max_p, ndc);
    }

    // Convert to 0..1 UV space
    let min_uv = min_p.xy * 0.5 + 0.5;
    let max_uv = max_p.xy * 0.5 + 0.5;
    
    // Invert Y for UV (if necessary, check coordinate systems)
    // WebGPU NDC: Y up. Texture UV: Y down? No, usually Y down in WGPU texture coords?
    // standard texture coords: (0,0) top-left.
    // NDC: (-1, -1) bottom-left?
    // WGSL textureLoad uses (0,0) top-left.
    // Let's assume standard mapping:
    // NDC x: -1..1 -> UV 0..1
    // NDC y: -1..1 -> UV 1..0 (flip Y)
    
    let min_uv_y = 1.0 - max_p.y * 0.5 - 0.5;
    let max_uv_y = 1.0 - min_p.y * 0.5 - 0.5;
    
    // Add a safety margin to the bounding box (e.g., 1-2% of screen)
    // This helps with temporal lag (camera moved, occluders shifted)
    // and prevents culling objects that are just on the edge.
    let margin = 0.01;
    let box_min = vec2<f32>(min_uv.x - margin, min_uv_y - margin);
    let box_max = vec2<f32>(max_uv.x + margin, max_uv_y + margin);
    
    // Check bounds (if fully off-screen, frustum cull should have caught it, but safe check)
    if (box_max.x < 0.0 || box_min.x > 1.0 || box_max.y < 0.0 || box_min.y > 1.0) {
        return false;
    }

    let size = (box_max - box_min) * vec2<f32>(textureDimensions(hiz_texture));
    let max_dim = max(size.x, size.y);
    
    // Select mip level such that the box covers roughly 2x2 pixels?
    // mip = log2(max_dim).
    // If box is 4 pixels wide, mip 2 (1 pixel).
    let mip = u32(ceil(log2(max(max_dim, 1.0))));
    let mip_clamped = min(mip, textureNumLevels(hiz_texture) - 1u);

    // Sample 4 texels from the Hi-Z buffer at that level to cover the box area
    // Since we picked a mip where the box is small, 4 samples (conservative) should cover it.
    // Wait, proper Hi-Z usually does specific sampling based on the box.
    // Simple approach: Sample one point at the center? No, dangerous.
    // Conservative: Sample the region covering the box.
    // If we use textureSampleLevel (linear), we might get interpolated depth which is NOT conservative max.
    // We MUST use textureLoad (point).
    
    // Let's scale UV to mip coordinates
    let mip_size = textureDimensions(hiz_texture, mip_clamped);
    let scale = vec2<f32>(mip_size);
    let min_tex = box_min * scale;
    let max_tex = box_max * scale;
    
    // We sample a grid of pixels covering the box?
    // Or just sample the calculated mip.
    // Since we ceil(log2), the box size in mip space is <= 1.0?
    // If max_dim was 100. log2(100) = 6.6 -> 7.
    // In mip 7, size is 100 / 128 = 0.78 pixels.
    // So it fits in 2x2 block.
    
    let x0 = u32(clamp(min_tex.x, 0.0, f32(mip_size.x - 1u)));
    let y0 = u32(clamp(min_tex.y, 0.0, f32(mip_size.y - 1u)));
    let x1 = u32(clamp(max_tex.x, 0.0, f32(mip_size.x - 1u)));
    let y1 = u32(clamp(max_tex.y, 0.0, f32(mip_size.y - 1u)));
    
    // Fetch depth from covered pixels
    var max_depth = 0.0;
    
    // Sampling 4 corners is usually enough if the box is <= 1 pixel in that mip
    // But let's loop to be safe (max 2x2 iterations if logic holds)
    for (var y = y0; y <= y1; y++) {
        for (var x = x0; x <= x1; x++) {
            let d = textureLoad(hiz_texture, vec2<u32>(x, y), mip_clamped).r;
            max_depth = max(max_depth, d);
        }
    }
    
    // Compare object nearest depth
    // Object Z is min_p.z (NDC 0..1 for WebGPU depth)
    let object_depth = min_p.z;
    
    // If object is FURTHER (larger Z) than the max_depth of the occluder, it is occluded.
    // Wait, in Hi-Z generation we stored MAX depth.
    // So 'max_depth' is the depth of the furthest thing in that region.
    // If object_depth > max_depth, it means the object is behind the furthest thing?
    // That doesn't mean it's occluded.
    // It means it's behind SOMETHING.
    // OCClUSION CULLING requires: We stored MIN depth (closest thing).
    // If object_depth > min_depth_of_occluders, then object is behind them.
    // Yes. Hi-Z for occlusion usually stores the FURTHEST visible point for conservative visibility?
    // No, standard Z-buffer: lower is closer.
    // If we want to know if something is hidden, we need to know the depth of the opaque geometry.
    // The Z-buffer contains the depth of the visible surface.
    // If we downsample, we want a conservative estimate of the depth of the surface.
    // If we have pixels A(0.1) and B(0.9). The geometry is at 0.1 and 0.9.
    // If we want to cull an object at 0.95.
    // If we store 0.1 (min), 0.95 > 0.1. We assume hidden? But B is 0.9. 0.95 > 0.9. Hidden behind B too.
    // What if object is at 0.5?
    // 0.5 > 0.1. But 0.5 < 0.9.
    // Is it hidden?
    // The pixel at B is open space until 0.9. So object at 0.5 is visible at B.
    // So the block is NOT fully opaque at 0.5.
    // So for a block to occlusion-cull an object at Z, the ENTIRE block must be closer than Z.
    // So we need the MAX depth of the block (the furthest point in the block).
    // If Z > MaxDepth(Block), then Z is behind the furthest point of the block.
    // Does that mean it is hidden?
    // Example: Block covers screen. Half is Z=0.1, Half is Z=0.9.
    // MaxDepth = 0.9.
    // Object at 0.95. 0.95 > 0.9. It is behind both halves. Occluded. YES.
    // Object at 0.5. 0.5 < 0.9. It is NOT behind the 0.9 half. Visible. YES.
    // So MAX depth is correct for "Is the whole block closer than object?".
    // Wait. If MaxDepth is 0.9. It means "The furthest thing in this block is at 0.9".
    // It implies there is stuff everywhere?
    // No, Z-buffer stores Cleared value (1.0) where empty.
    // If empty, MaxDepth = 1.0.
    // Object at 0.95. 0.95 < 1.0. Not occluded. Correct.
    
    // So yes, MAX reduction is correct.
    // And condition: object_depth > max_depth => Occluded.
    // But we need a small bias to prevent z-fighting flickering.
    // Increased bias to be safer
    return object_depth > (max_depth + 0.002); 
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= arrayLength(&all_instances)) {
        return;
    }

    let instance = all_instances[index];
    let frustum = frustum_from_view_proj(camera.view_proj);
    let aabb = get_aabb_from_instance(instance.model);

    // 1. Frustum Culling
    if (intersects_aabb(frustum, aabb)) {
        // 2. Occlusion Culling (Hi-Z)
        // Disabled for now
        // if (!is_occluded(aabb)) {
            let out_index = atomicAdd(&draw_cmd.instance_count, 1u);
            visible_instances[out_index] = instance;
        // }
    }
}
