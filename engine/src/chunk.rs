use crate::cube::Cube;
use bytemuck::{Pod, Zeroable};
use glam::{IVec3, UVec3};
use rustc_hash::{FxBuildHasher, FxHashMap};
use wgpu::util::DeviceExt;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Pod, Zeroable)]
pub struct VoxelGpu {
    color: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct ChunkMetaGpu {
    pub position: IVec3,
    pub offset: u32,
}

#[derive(Debug, Clone)]
pub struct World {
    pub voxel_buffer: wgpu::Buffer,
    pub chunk_meta_buffer: wgpu::Buffer,
    pub chunk_count: usize,
}

impl World {
    pub fn new(device: &wgpu::Device, cubes: &[Cube]) -> Self {
        let temp = Self::group_cubes(cubes);
        let chunk_number = temp.len();
        let total_voxels = temp.len() * CHUNK_VOLUME;

        let mut voxels_data: Vec<VoxelGpu> = Vec::with_capacity(total_voxels);
        let mut chunks_data = Vec::with_capacity(chunk_number);

        for (chunk_pos, voxels) in temp {
            let offset = u32::try_from(voxels_data.len()).unwrap();
            voxels_data.extend_from_slice(&voxels);
            chunks_data.push(ChunkMetaGpu {
                offset,
                position: chunk_pos,
            });
        }

        let voxel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ETIB Voxel Buffer"),
            usage: wgpu::BufferUsages::STORAGE,
            contents: bytemuck::cast_slice(&voxels_data),
        });

        let chunk_meta_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ETIB Chunk Meta Buffer"),
            usage: wgpu::BufferUsages::STORAGE,
            contents: bytemuck::cast_slice(&chunks_data),
        });

        Self {
            voxel_buffer,
            chunk_meta_buffer,
            chunk_count: chunk_number,
        }
    }

    pub const fn cube_index(pos_in_chunk: UVec3) -> usize {
        pos_in_chunk.x as usize
            + pos_in_chunk.y as usize * CHUNK_SIZE
            + pos_in_chunk.z as usize * CHUNK_SIZE * CHUNK_SIZE
    }

    pub fn group_cubes(cubes: &[Cube]) -> FxHashMap<IVec3, Vec<VoxelGpu>> {
        let estimated_count = cubes.len() / CHUNK_SIZE;
        let mut temp = FxHashMap::with_capacity_and_hasher(estimated_count, FxBuildHasher);

        for cube in cubes {
            let chunk_pos = cube
                .position
                .div_euclid(IVec3::splat(i32::try_from(CHUNK_SIZE).unwrap()));
            let cube_pos_in_chunk = cube
                .position
                .rem_euclid(IVec3::splat(i32::try_from(CHUNK_SIZE).unwrap()))
                .as_uvec3();

            temp.entry(chunk_pos)
                .or_insert_with(|| vec![VoxelGpu { color: 0 }; CHUNK_VOLUME])
                [Self::cube_index(cube_pos_in_chunk)] = VoxelGpu { color: cube.color };
        }

        temp
    }
}
