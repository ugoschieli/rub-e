/// GPU-compatible chunk metadata.
///
/// Flat layout (48 bytes, `repr(C)`) that matches the WGSL `ChunkRaw` struct
/// in `chunk_cull.wgsl` byte-for-byte. Avoids `vec3` alignment pitfalls.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkRaw {
    /// World-space AABB minimum corner (bytes 0–11).
    pub aabb_min: [f32; 3],
    #[doc(hidden)]
    pub _pad0: f32,
    /// World-space AABB maximum corner (bytes 16–27).
    pub aabb_max: [f32; 3],
    /// First cube index in the static `all_instances` buffer (bytes 28–31).
    pub start_idx: u32,
    /// Number of cubes in this chunk (bytes 32–35).
    pub count: u32,
    #[doc(hidden)]
    pub _pad1: [u32; 3],
}

// ---------------------------------------------------------------------------
// Chunk-level culling pass
// ---------------------------------------------------------------------------

/// GPU frustum culling at chunk granularity.
///
/// One compute thread per chunk. Tests the chunk's pre-computed AABB and writes
/// `chunk_visible[]` (1 = visible, 0 = culled).  The per-cube [`CullingPass`]
/// reads that buffer to skip every cube in a culled chunk.
pub struct ChunkCullingPass {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ChunkCullingPass {
    /// Create the chunk culling pipeline.
    pub fn new(device: &wgpu::Device, camera_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Chunk Culling Bind Group Layout"),
                entries: &[
                    // binding 0: chunks (read-only)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // binding 1: chunk_visible (read/write)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Chunk Culling Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/chunk_cull.wgsl").into(),
            ),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Chunk Culling Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout, &bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Chunk Culling Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    /// Build a bind group for the given chunk + visibility buffers.
    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        chunks_buffer: &wgpu::Buffer,
        chunk_visible_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Chunk Culling Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: chunks_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: chunk_visible_buffer.as_entire_binding(),
                },
            ],
        })
    }

    /// Dispatch the chunk culling compute shader.
    pub fn cull(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        camera_bind_group: &wgpu::BindGroup,
        chunk_bind_group: &wgpu::BindGroup,
        num_chunks: u32,
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Chunk Culling Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, camera_bind_group, &[]);
        pass.set_bind_group(1, chunk_bind_group, &[]);
        let workgroups = (num_chunks + 63) / 64;
        pass.dispatch_workgroups(workgroups, 1, 1);
    }
}

// ---------------------------------------------------------------------------
// Per-cube culling pass
// ---------------------------------------------------------------------------

/// GPU per-cube frustum culling compute pass.
///
/// Reads `chunk_visible[]` (written by [`ChunkCullingPass`]) to skip cubes
/// that belong to a fully-culled chunk, then tests each surviving cube's own
/// AABB and compacts visible instances into `visible_instances`.
pub struct CullingPass {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl CullingPass {
    /// Create a new culling pass.
    pub fn new(device: &wgpu::Device, camera_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Culling Bind Group Layout"),
                entries: &[
                    // binding 0: all_instances (read-only)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // binding 1: visible_instances (read/write)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // binding 2: indirect draw buffer (read/write)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // binding 3: cube_chunk_ids (read-only)
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // binding 4: chunk_visible (read-only — written by ChunkCullingPass)
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Culling Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/cull.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Culling Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout, &bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Culling Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    /// Build a bind group for a specific set of buffers.
    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        all_instances: &wgpu::Buffer,
        visible_instances: &wgpu::Buffer,
        indirect_buffer: &wgpu::Buffer,
        cube_chunk_ids: &wgpu::Buffer,
        chunk_visible: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Culling Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: all_instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: visible_instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: indirect_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: cube_chunk_ids.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: chunk_visible.as_entire_binding(),
                },
            ],
        })
    }

    /// Dispatch the per-cube culling compute shader.
    pub fn cull(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        camera_bind_group: &wgpu::BindGroup,
        culling_bind_group: &wgpu::BindGroup,
        total_instance_count: u32,
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Culling Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, camera_bind_group, &[]);
        pass.set_bind_group(1, culling_bind_group, &[]);
        let workgroups = (total_instance_count + 63) / 64;
        pass.dispatch_workgroups(workgroups, 1, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_raw_layout() {
        let chunk = ChunkRaw {
            aabb_min: [-1.0, -1.0, -1.0],
            _pad0: 0.0,
            aabb_max: [1.0, 1.0, 1.0],
            start_idx: 100,
            count: 50,
            _pad1: [0; 3],
        };
        
        assert_eq!(chunk.start_idx, 100);
        assert_eq!(chunk.count, 50);
        assert_eq!(chunk.aabb_min, [-1.0, -1.0, -1.0]);
        assert_eq!(std::mem::size_of::<ChunkRaw>(), 48); // 12 * 4 bytes
    }

    #[test]
    fn test_culling_pipelines_headless() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));
        
        if let Ok(adapter) = adapter {
            let (device, _) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
            
            // Need a dummy layout for camera
            let entries = [
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ];
            let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &entries,
            });
            
            let chunk_pass = ChunkCullingPass::new(&device, &camera_layout);
            let _cull_pass = CullingPass::new(&device, &camera_layout);
            
            // Ensure we created the passes without panicking
            assert!(true);
        }
    }
}
