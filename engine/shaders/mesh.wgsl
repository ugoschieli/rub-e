const CHUNK_SIZE: u32 = 32u;
const CHUNK_VOLUME: u32 = 32768u; // 32 * 32 * 32
const VOXELS_PER_THREAD: u32 = 512u; // CHUNK_VOLUME / workgroup_size(64)

struct ChunkMeta {
    position: vec3<i32>,
    offset: u32,
}

struct VoxelGpu {
    color: u32,
}

struct VisibleChunks {
    count: u32,
    indices: array<u32>,
}

struct Face {
    position: vec3<i32>,
    direction: u32,
    color: u32,
}

// Mirrors wgpu::util::DrawIndirectArgs — also used directly as the draw_indirect source.
// vertex_count is incremented atomically; the rest are constant after buffer init.
struct DrawArgs {
    vertex_count: atomic<u32>,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}

@group(0) @binding(0) var<storage, read> visible_chunks: VisibleChunks;
@group(0) @binding(1) var<storage, read> chunks: array<ChunkMeta>;
@group(0) @binding(2) var<storage, read> voxels: array<VoxelGpu>;
@group(0) @binding(3) var<storage, read_write> draw_args: DrawArgs;
@group(0) @binding(4) var<storage, read_write> faces: array<Face>;

// One workgroup per visible chunk (dispatched indirectly).
// 64 threads each iterate over 512 consecutive voxels to cover the full CHUNK_VOLUME.
@compute @workgroup_size(64)
fn main(
    @builtin(workgroup_id) wg_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {
    let chunk_slot = wg_id.x;
    if chunk_slot >= visible_chunks.count {
        return;
    }

    let chunk = chunks[visible_chunks.indices[chunk_slot]];
    let start_voxel = local_id.x * VOXELS_PER_THREAD;

    for (var i = 0u; i < VOXELS_PER_THREAD; i++) {
        let vi = start_voxel + i;

        let voxel = voxels[chunk.offset + vi];
        if voxel.color == 0u { continue; }

        let x = vi % CHUNK_SIZE;
        let y = (vi / CHUNK_SIZE) % CHUNK_SIZE;
        let z = vi / (CHUNK_SIZE * CHUNK_SIZE);
        let world_pos = chunk.position * i32(CHUNK_SIZE) + vec3<i32>(i32(x), i32(y), i32(z));

        // Claim 6 consecutive face slots; vertex_count tracks total vertices (6 per face).
        let base = atomicAdd(&draw_args.vertex_count, 36u) / 6u;
        for (var d = 0u; d < 6u; d++) {
            faces[base + d] = Face(world_pos, d, voxel.color);
        }
    }
}
