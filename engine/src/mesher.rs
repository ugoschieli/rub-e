use crate::{
    constants::{CHUNK_SIZE_1, CHUNK_SIZE_3, CHUNK_SIZE_P},
    cube::VoxelGpu,
};
use bytemuck::{Pod, Zeroable};
use glam::{UVec3, Vec3, uvec3};
use std::collections::HashMap;
use std::hash::BuildHasher;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct FaceGpu {
    position: Vec3,
    width: u32,
    height: u32,
    direction: u32,
    color: u32,
    _pad: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Face {
    pub position: Vec3,
    pub width: u32,
    pub height: u32,
    pub direction: FaceDirection,
    pub color: UVec3,
}

impl Face {
    pub const fn new(
        position: Vec3,
        width: u32,
        height: u32,
        direction: FaceDirection,
        color: UVec3,
    ) -> Self {
        Self {
            position,
            width,
            height,
            direction,
            color,
        }
    }

    pub const fn to_gpu(&self) -> FaceGpu {
        FaceGpu {
            position: self.position,
            width: self.width,
            height: self.height,
            direction: self.direction.to_gpu(),
            color: pack_color(self.color),
            _pad: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FaceDirection {
    Right,    // +X = 0
    Left,     // -X = 1
    Forward,  // +Y = 2
    Backward, // -Y = 3
    Up,       // +Z = 4
    Down,     // -Z = 5
}

impl FaceDirection {
    pub const fn to_gpu(&self) -> u32 {
        match self {
            Self::Right => 0,
            Self::Left => 1,
            Self::Forward => 2,
            Self::Backward => 3,
            Self::Up => 4,
            Self::Down => 5,
        }
    }
}

pub const fn pack_color(color: UVec3) -> u32 {
    color.x | (color.y << 10) | (color.z << 20) // packed color (10 bit for each channel)
}

pub const fn unpack_color(color: u32) -> UVec3 {
    let r = color & 1023;
    let g = (color >> 10) & 1023;
    let b = (color >> 20) & 1023;

    UVec3::new(r, g, b)
}

#[derive(Debug, PartialEq, Eq)]
struct GreedyQuad {
    u: u8,
    v: u8,
    w: u8,
    h: u8,
}

impl GreedyQuad {
    const fn new(u: u8, v: u8, w: u8, h: u8) -> Self {
        Self { u, v, w, h }
    }
}

pub fn mesh_chunk(chunk: &[VoxelGpu; CHUNK_SIZE_3]) -> Vec<Face> {
    // solid binary for each x,y,z axis (3)
    let mut axis_cols: Box<[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3]> = bytemuck::zeroed_box();
    // the cull mask to perform greedy slicing, based on solids on previous axis_cols
    let mut col_face_masks: Box<[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6]> = bytemuck::zeroed_box();

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

        for (color, axis_plane) in block_data {
            for (axis_pos, plane) in axis_plane {
                let quads_from_axis = greedy_mesh_binary_plane(plane);
                for q in quads_from_axis {
                    let w = axis_pos;

                    let voxel_pos = match axis {
                        0 | 1 => uvec3(w, u32::from(q.v), u32::from(q.u)), // left, right
                        2 | 3 => uvec3(u32::from(q.u), w, u32::from(q.v)), // forward, back
                        _ => uvec3(u32::from(q.u), u32::from(q.v), w),     // down,up
                    };

                    // For X faces the greedy plane is transposed relative to the
                    // renderer's scale convention (width scales local X -> world Y,
                    // height scales local Z -> world Z), so width/height must be
                    // swapped. Y and Z faces already align.
                    let (width, height) = match axis {
                        0 | 1 => (u32::from(q.h), u32::from(q.w)),
                        _ => (u32::from(q.w), u32::from(q.h)),
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
                }
            }
        }
    }
    faces
}

pub const fn chunk_index(pos: UVec3) -> usize {
    pos.x as usize + CHUNK_SIZE_1 * (pos.y as usize + CHUNK_SIZE_1 * pos.z as usize)
}

// fn chunk_pos(i: u32) -> UVec3 {
//     let cs = u32::try_from(CHUNK_SIZE_1).unwrap();
//     let cs2 = u32::try_from(CHUNK_SIZE_2).unwrap();
//     UVec3::new(i % cs, i.div_euclid(cs) % cs, i.div_euclid(cs2))
// }

const fn add_voxel_to_axis_cols(
    b: VoxelGpu,
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

fn fill_axis_cols(
    chunk: &[VoxelGpu; CHUNK_SIZE_3],
    axis_cols: &mut [[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3],
) {
    for z in 0..CHUNK_SIZE_1 {
        for y in 0..CHUNK_SIZE_1 {
            for x in 0..CHUNK_SIZE_1 {
                let i = chunk_index(uvec3(
                    u32::try_from(x).unwrap(),
                    u32::try_from(y).unwrap(),
                    u32::try_from(z).unwrap(),
                ));
                add_voxel_to_axis_cols(chunk[i], x + 1, y + 1, z + 1, axis_cols);
            }
        }
    }
}

fn cull_axis_cols(
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

type BinaryPlane = [u64; CHUNK_SIZE_1];
type PlanesByType<S2> = HashMap<u32, BinaryPlane, S2>;
type PlanesByAxis<S1, S2> = HashMap<u32, PlanesByType<S2>, S1>;
type BinaryPlaneData<S1, S2> = [PlanesByAxis<S1, S2>; 6];
fn build_binary_planes<S1: BuildHasher, S2: Default + BuildHasher>(
    col_face_masks: &[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6],
    chunk: &[VoxelGpu; CHUNK_SIZE_3],
    data: &mut BinaryPlaneData<S1, S2>,
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
                    let u = u32::try_from(u).unwrap();
                    let v = u32::try_from(v).unwrap();
                    let voxel_pos = match axis {
                        0 | 1 => uvec3(w, v, u), // left, right
                        2 | 3 => uvec3(u, w, v), // forward, back
                        _ => uvec3(u, v, w),     // down,up
                    };

                    let current_voxel = chunk[chunk_index(voxel_pos)];
                    // we can only greedy mesh same block types
                    let block_hash = current_voxel.color;
                    let data = data[axis]
                        .entry(block_hash)
                        .or_default()
                        .entry(w)
                        .or_insert([0; CHUNK_SIZE_1]);
                    data[u as usize] |= 1 << v;
                }
            }
        }
    }
}

fn greedy_mesh_binary_plane(mut data: [u64; CHUNK_SIZE_1]) -> Vec<GreedyQuad> {
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
            quads.push(GreedyQuad::new(
                u8::try_from(u).unwrap(),
                u8::try_from(v).unwrap(),
                u8::try_from(w).unwrap(),
                u8::try_from(h).unwrap(),
            ));
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
        let mut chunk: Box<[VoxelGpu; CHUNK_SIZE_3]> = bytemuck::zeroed_box();
        let mut axis_cols: Box<[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3]> = bytemuck::zeroed_box();

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
        let mut chunk: Box<[VoxelGpu; CHUNK_SIZE_3]> = bytemuck::zeroed_box();
        let mut axis_cols: Box<[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3]> = bytemuck::zeroed_box();
        let mut col_face_masks: Box<[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6]> =
            bytemuck::zeroed_box();

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
        let mut chunk: Box<[VoxelGpu; CHUNK_SIZE_3]> = bytemuck::zeroed_box();
        let mut axis_cols: Box<[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3]> = bytemuck::zeroed_box();
        let mut col_face_masks: Box<[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6]> =
            bytemuck::zeroed_box();

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
        let mut chunk: Box<[VoxelGpu; CHUNK_SIZE_3]> = bytemuck::zeroed_box();
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
