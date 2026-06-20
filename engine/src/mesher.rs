use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use glam::{UVec3, uvec3};

use crate::renderer::static_renderer::{Face, FaceDirection, unpack_color};
use crate::{
    constants::{CHUNK_SIZE_1, CHUNK_SIZE_2, CHUNK_SIZE_3, CHUNK_SIZE_P},
    cube::VoxelGpu,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
/// The Face data uploaded to the GPU for vertex pulling
pub struct FaceGpu {
    /// The packed data of a Face:
    /// The 18 first bits encode the chunk-local position (6 bits for each axis)
    /// The 12 next bits encode the width and height (6 bits each)
    /// The 3 next bits encode the normal direction (6 possible values)
    /// The 30 next bits encode the RGB color (10 bits for each channel)
    /// Total 63 bits
    pub data: u64,
}

#[derive(Debug, PartialEq)]
pub struct GreedyQuad {
    pub u: u8,
    pub v: u8,
    pub w: u8,
    pub h: u8,
}

impl GreedyQuad {
    pub fn new(u: u32, v: u32, w: u32, h: u32) -> Self {
        Self {
            u: u as u8,
            v: v as u8,
            w: w as u8,
            h: h as u8,
        }
    }
}

pub fn mesh_chunk(chunk: &[VoxelGpu; CHUNK_SIZE_3]) -> Vec<Face> {
    // solid binary for each x,y,z axis (3)
    let mut axis_cols = [[[0u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3];
    // the cull mask to perform greedy slicing, based on solids on previous axis_cols
    let mut col_face_masks = [[[0u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6];

    // iterate over all voxels in the chunk to fill axis_cols
    fill_axis_cols(chunk, &mut axis_cols);
    // neighbors are skipped for now
    // face culling
    cull_axis_cols(&axis_cols, &mut col_face_masks);

    // greedy meshing planes for every axis (6)
    // key(block color) -> HashMap<axis(0-32), binary_plane>
    // note: don't ask me how this isn't a massive bottleneck.
    //  might become an issue in the future, when there are more block types.
    //  consider using a single hashmap with key (axis, block_hash, y).
    let mut data: [HashMap<u32, HashMap<u32, [u64; CHUNK_SIZE_1]>>; 6];
    data = [
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    ];

    // find faces and build binary planes based on the voxel block etc...
    build_binary_planes(&col_face_masks, chunk, &mut data);

    let mut faces: Vec<Face> = vec![];
    for (axis, block_data) in data.into_iter().enumerate() {
        // col_face_masks index 2*axis+0 isolates the positive-side face
        // (air on the higher neighbor) and 2*axis+1 the negative-side face.
        let direction = match axis {
            0 => FaceDirection::Right,
            1 => FaceDirection::Left,
            2 => FaceDirection::Forward,
            3 => FaceDirection::Backward,
            4 => FaceDirection::Up,
            _ => FaceDirection::Down,
        };

        for (color, axis_plane) in block_data.into_iter() {
            for (axis_pos, plane) in axis_plane.into_iter() {
                let quads_from_axis = greedy_mesh_binary_plane(plane);
                quads_from_axis.into_iter().for_each(|q| {
                    let w = axis_pos;

                    let voxel_pos = match axis {
                        0 | 1 => uvec3(w, q.v as u32, q.u as u32), // left, right
                        2 | 3 => uvec3(q.u as u32, w, q.v as u32), // forward, back
                        _ => uvec3(q.u as u32, q.v as u32, w),     // down,up
                    };

                    // For X faces the greedy plane is transposed relative to the
                    // renderer's scale convention (width scales local X -> world Y,
                    // height scales local Z -> world Z), so width/height must be
                    // swapped. Y and Z faces already align.
                    let (width, height) = match axis {
                        0 | 1 => (q.h as u32, q.w as u32),
                        _ => (q.w as u32, q.h as u32),
                    };

                    // The Left/Forward/Down face matrices are pure rotations that
                    // mirror an extent axis, so the quad lands at negative offsets.
                    // Shift position by the quad size to bring it back onto the
                    // voxel (and +1 along the face normal where the matrix no longer
                    // carries that translation).
                    let position_offset = match axis {
                        1 => uvec3(0, width, 0),  // Left:    rot_z(-90) mirrors width onto -Y
                        2 => uvec3(width, 1, 0),  // Forward: rot_z(180) mirrors width onto -X
                        5 => uvec3(0, height, 0), // Down:    rot_x(90)  mirrors height onto -Y
                        _ => uvec3(0, 0, 0),
                    };

                    faces.push(Face {
                        position: (voxel_pos + position_offset).as_vec3(),
                        color: unpack_color(color),
                        width,
                        height,
                        direction,
                    });
                });
            }
        }
    }
    faces
}

pub const fn chunk_index(pos: UVec3) -> usize {
    pos.x as usize + CHUNK_SIZE_1 * (pos.y as usize + CHUNK_SIZE_1 * pos.z as usize)
}

pub const fn chunk_pos(i: usize) -> UVec3 {
    UVec3::new(
        (i % CHUNK_SIZE_1) as u32,
        (i.div_euclid(CHUNK_SIZE_1) % CHUNK_SIZE_1) as u32,
        i.div_euclid(CHUNK_SIZE_2) as u32,
    )
}

const fn add_voxel_to_axis_cols(
    b: &VoxelGpu,
    x: usize,
    y: usize,
    z: usize,
    axis_cols: &mut [[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3],
) {
    if b.color != 0 {
        // z,y - x axis
        axis_cols[0][y][z] |= 1u64 << x as u64;
        // x,z - y axis
        axis_cols[1][z][x] |= 1u64 << y as u64;
        // x,y - z axis
        axis_cols[2][y][x] |= 1u64 << z as u64;
    }
}

pub fn fill_axis_cols(
    chunk: &[VoxelGpu; CHUNK_SIZE_3],
    axis_cols: &mut [[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3],
) {
    for z in 0..CHUNK_SIZE_1 {
        for y in 0..CHUNK_SIZE_1 {
            for x in 0..CHUNK_SIZE_1 {
                let i = chunk_index(uvec3(x as u32, y as u32, z as u32));
                add_voxel_to_axis_cols(&chunk[i], x + 1, y + 1, z + 1, axis_cols);
            }
        }
    }
}

pub fn cull_axis_cols(
    axis_cols: &[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3],
    col_face_masks: &mut [[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6],
) {
    for axis in 0..3 {
        for v in 0..CHUNK_SIZE_P {
            for u in 0..CHUNK_SIZE_P {
                let col = axis_cols[axis][v][u];
                // positive side: solid here, air at the higher neighbor (bit + 1)
                col_face_masks[2 * axis][v][u] = col & !(col >> 1);
                // negative side: solid here, air at the lower neighbor (bit - 1)
                col_face_masks[2 * axis + 1][v][u] = col & !(col << 1);
            }
        }
    }
}

pub fn build_binary_planes(
    col_face_masks: &[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6],
    chunk: &[VoxelGpu; CHUNK_SIZE_3],
    data: &mut [HashMap<u32, HashMap<u32, [u64; CHUNK_SIZE_1]>>; 6],
) {
    for axis in 0..6 {
        for v in 0..CHUNK_SIZE_1 {
            for u in 0..CHUNK_SIZE_1 {
                // skip padded by adding 1(for x padding) and (z+1) for (z padding)
                let mut col = col_face_masks[axis][v + 1][u + 1];

                // removes the right most padding value, because it's invalid
                col >>= 1;
                // removes the left most padding value, because it's invalid
                col &= !(1 << CHUNK_SIZE_1 as u64);

                while col != 0 {
                    let w = col.trailing_zeros();
                    // clear least significant set bit
                    col &= col - 1;

                    // get the voxel position based on axis
                    let voxel_pos = match axis {
                        0 | 1 => uvec3(w, v as u32, u as u32), // left, right
                        2 | 3 => uvec3(u as u32, w, v as u32), // forward, back
                        _ => uvec3(u as u32, v as u32, w),     // down,up
                    };

                    let current_voxel = chunk[chunk_index(voxel_pos)];
                    // we can only greedy mesh same block types
                    let block_hash = current_voxel.color;
                    let data = data[axis]
                        .entry(block_hash)
                        .or_default()
                        .entry(w)
                        .or_insert([0; CHUNK_SIZE_1]);
                    data[u] |= 1 << v;
                }
            }
        }
    }
}

pub fn greedy_mesh_binary_plane(mut data: [u64; CHUNK_SIZE_1]) -> Vec<GreedyQuad> {
    let mut quads = vec![];
    for u in 0..CHUNK_SIZE_1 {
        let mut v = 0;
        while v < CHUNK_SIZE_1 {
            // find first solid, "air/zero's" could be first so skip
            v += (data[u] >> v).trailing_zeros() as usize;
            if v >= CHUNK_SIZE_1 {
                // reached top
                continue;
            }
            let h = (data[u] >> v).trailing_ones();
            // convert height 'num' to positive bits repeated 'num' times aka:
            // 1 = 0b1, 2 = 0b11, 4 = 0b1111
            let h_as_mask = u64::checked_shl(1, h).map_or(!0, |v| v - 1);
            let mask = h_as_mask << v;
            // grow horizontally
            let mut w = 1;
            while u + w < CHUNK_SIZE_1 {
                // fetch bits spanning height, in the next row
                let next_row_h = (data[u + w] >> v) & h_as_mask;
                if next_row_h != h_as_mask {
                    break; // can no longer expand horizontally
                }

                // nuke the bits we expanded into
                data[u + w] &= !mask;

                w += 1;
            }
            quads.push(GreedyQuad::new(u as u32, v as u32, w as u32, h));
            v += h as usize;
        }
    }
    quads
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_axis_cols_test() {
        let mut chunk = [VoxelGpu { color: 0 }; CHUNK_SIZE_3];
        let mut axis_cols = [[[0u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3];

        chunk[chunk_index(uvec3(0, 0, 0))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(1, 0, 0))] = VoxelGpu { color: 1023 };

        chunk[chunk_index(uvec3(0, 1, 0))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(0, 2, 0))] = VoxelGpu { color: 1023 };

        fill_axis_cols(&chunk, &mut axis_cols);

        // X-axis
        assert_eq!(axis_cols[0][1][1], 0b110);
        assert_eq!(axis_cols[0][2][1], 0b10);
        assert_eq!(axis_cols[0][3][1], 0b10);

        // Y-axis
        assert_eq!(axis_cols[1][1][1], 0b1110);
        assert_eq!(axis_cols[1][1][2], 0b10);
    }

    #[test]
    fn cull_axis_cols_test() {
        let mut chunk = [VoxelGpu { color: 0 }; CHUNK_SIZE_3];
        let mut axis_cols = [[[0u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3];
        let mut col_face_masks = [[[0u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6];

        chunk[chunk_index(uvec3(0, 0, 0))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(1, 0, 0))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(2, 0, 0))] = VoxelGpu { color: 1023 };

        fill_axis_cols(&chunk, &mut axis_cols);
        cull_axis_cols(&axis_cols, &mut col_face_masks);

        // +X and -X
        assert_eq!(col_face_masks[1][1][1], 0b10);
        assert_eq!(col_face_masks[0][1][1], 0b1000);

        // +Y
        assert_eq!(col_face_masks[2][1][1], 0b10);
        assert_eq!(col_face_masks[2][1][2], 0b10);
        assert_eq!(col_face_masks[2][1][3], 0b10);
    }

    #[test]
    fn build_binary_planes_test() {
        let mut chunk = [VoxelGpu { color: 0 }; CHUNK_SIZE_3];
        let mut axis_cols = [[[0u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3];
        let mut col_face_masks = [[[0u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6];

        chunk[chunk_index(uvec3(1, 0, 1))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(1, 0, 2))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(2, 0, 1))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(2, 0, 2))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(2, 0, 3))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(3, 0, 2))] = VoxelGpu { color: 1023 };

        let mut data: [HashMap<u32, HashMap<u32, [u64; CHUNK_SIZE_1]>>; 6];
        data = [
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        ];

        fill_axis_cols(&chunk, &mut axis_cols);
        cull_axis_cols(&axis_cols, &mut col_face_masks);
        build_binary_planes(&col_face_masks, &chunk, &mut data);

        let plane = data[2].get(&1023).unwrap().get(&0).unwrap();

        assert_eq!(plane[1], 0b0110);
        assert_eq!(plane[2], 0b1110);
        assert_eq!(plane[3], 0b0100);
    }

    #[test]
    fn greedy_mesh_binary_plane_tests() {
        let mut data = [0; CHUNK_SIZE_1];
        data[1] = 0b0110;
        data[2] = 0b1110;
        data[3] = 0b0100;

        let quads = greedy_mesh_binary_plane(data);
        assert_eq!(
            quads,
            vec![
                GreedyQuad::new(1, 1, 2, 2,),
                GreedyQuad::new(2, 3, 1, 1,),
                GreedyQuad::new(3, 2, 1, 1,),
            ]
        );
    }

    #[test]
    fn mesh_chunk_test() {
        let mut chunk = [VoxelGpu { color: 0 }; CHUNK_SIZE_3];
        chunk[chunk_index(uvec3(0, 0, 0))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(1, 0, 0))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(2, 0, 0))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(0, 0, 1))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(1, 0, 1))] = VoxelGpu {
            color: 2u32.pow(20) - 1,
        };
        chunk[chunk_index(uvec3(2, 0, 1))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(0, 0, 2))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(1, 0, 2))] = VoxelGpu { color: 1023 };
        chunk[chunk_index(uvec3(2, 0, 2))] = VoxelGpu { color: 1023 };

        let faces = mesh_chunk(&chunk);
        assert_eq!(faces.len(), 14);
    }
}
