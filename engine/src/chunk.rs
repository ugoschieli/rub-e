use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Pod, Zeroable)]
pub struct VoxelGpu {
    pub color: u32,
}
