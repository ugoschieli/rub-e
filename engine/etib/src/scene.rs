//! High-level scene renderer: static models, dynamic models, skybox, and HDR.
//!
//! [`Scene`] is the single rendering entry point. It owns the render pipeline,
//! GPU buffers, frustum culling, dynamic model management, the HDR intermediate
//! texture, and the optional skybox. The caller only needs to load models,
//! update transforms, and set the current scene with [`scene`](crate::game::Game::scene).

use std::collections::HashMap;

use anyhow::Result;
use cgmath::{Matrix4, Vector4};
use wgpu::util::DeviceExt;

use crate::camera::Camera;
use crate::cube::{
    Cube, CubeRaw, ChunkCullingPass, ChunkRaw, CullingPass, DynamicModel, DynamicScene, INDICES,
    ModelCube, VERTICES,
};
use crate::gfx::Gfx;
use crate::hdr::{HdrLoader, HdrPipeline, TonemappingMode};
use crate::{EngineContext, Vertex};
use etib_core::bindgroup::BindGroupBuilder;
use etib_core::pipeline::Pipeline;
use etib_core::shader::Shader;

const CUBE_SHADER: &str = include_str!("shaders/shader.wgsl");
const SKYBOX_SHADER: &str = include_str!("shaders/skybox.wgsl");

/// Each static chunk groups cubes in a 32×32 XZ tile (Y is unbounded).
const CHUNK_SIZE: f32 = 32.0;

struct SkyboxData {
    pipeline: Pipeline,
    bind_group: etib_core::bindgroup::BindGroup,
}

/// Manages the full rendering of static models, dynamic models, skybox, and HDR.
///
/// Static models are baked in at construction time and never change. Dynamic
/// models can be added, removed, and transformed at runtime — each is a
/// multi-cube group from a `.model` file that moves as a rigid body.
///
/// # Frame loop
///
/// ```text
/// // A single call handles culling, the render pass, and HDR tonemapping:
/// scene.render(&mut encoder, &gfx, &swapchain_view, &camera.bind_group.bind_group);
/// ```
pub struct Scene {
    // Shared cube geometry
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    pipeline: Pipeline,

    // Static instances — sorted by chunk, baked in at construction
    // Kept alive so the GPU buffer remains valid for the culling bind group.
    #[allow(dead_code)]
    all_instances_buffer: wgpu::Buffer,
    visible_instances_buffer: wgpu::Buffer,
    indirect_buffer: wgpu::Buffer,
    total_instance_count: u32,

    // Chunk-level culling (pass 1)
    chunk_cull_pass: ChunkCullingPass,
    chunk_culling_bind_group: wgpu::BindGroup,
    num_chunks: u32,
    #[allow(dead_code)]
    chunks_buffer: wgpu::Buffer,
    #[allow(dead_code)]
    chunk_visible_buffer: wgpu::Buffer,

    // Per-cube culling (pass 2)
    cull_pass: CullingPass,
    culling_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    cube_chunk_ids_buffer: wgpu::Buffer,

    /// Dynamic instances — rebuilt every frame
    dynamic_scene: DynamicScene,

    /// Optional skybox
    skybox: Option<SkyboxData>,

    /// HDR intermediate texture + tonemap pipeline
    hdr: HdrPipeline,

    /// The camera scene
    pub camera: Camera,
}

impl Scene {
    /// Create a new scene.
    ///
    /// `static_cubes` is the flat list of all cubes for the fixed geometry.
    /// Use [`load_model`](crate::cube::load_model) to produce it.
    ///
    /// `dynamic_max_instances` caps the **total number of cubes** across all
    /// live dynamic models and pre-allocates the dynamic GPU buffer accordingly.
    #[cfg(not(tarpaulin_include))]
    pub fn new(
        ctx: &EngineContext,
        camera: Camera,
        static_cubes: &[ModelCube],
        dynamic_max_instances: usize,
    ) -> Self {
        let gfx = &ctx.gfx;
        let device = gfx.device();

        let tonemap_mode = if gfx.is_hdr_active {
            TonemappingMode::Hdr
        } else {
            TonemappingMode::Sdr
        };
        let hdr = HdrPipeline::new(
            device,
            &gfx.surface_config,
            tonemap_mode,
            gfx.peak_brightness_nits,
        );
        let target_format = hdr.format();

        // Shared unit-cube geometry
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // ---------------------------------------------------------------
        // CPU chunking: sort cubes into 32×32 XZ tiles and compute AABBs.
        // ---------------------------------------------------------------

        // Map chunk key → indices into `static_cubes`
        let mut chunk_map: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (i, cube) in static_cubes.iter().enumerate() {
            let cx = (cube.position.x / CHUNK_SIZE).floor() as i32;
            let cz = (cube.position.z / CHUNK_SIZE).floor() as i32;
            chunk_map.entry((cx, cz)).or_default().push(i);
        }

        // Deterministic chunk order
        let mut chunk_keys: Vec<(i32, i32)> = chunk_map.keys().copied().collect();
        chunk_keys.sort_unstable();

        let mut instance_data: Vec<CubeRaw> = Vec::with_capacity(static_cubes.len());
        let mut cube_chunk_ids: Vec<u32> = Vec::with_capacity(static_cubes.len());
        let mut chunks_raw: Vec<ChunkRaw> = Vec::with_capacity(chunk_keys.len());

        for (chunk_idx, key) in chunk_keys.iter().enumerate() {
            let indices = &chunk_map[key];
            let start_idx = instance_data.len() as u32;
            let count = indices.len() as u32;

            // Tight AABB over all cube faces in this chunk
            let mut min = [f32::MAX; 3];
            let mut max = [f32::MIN; 3];
            for &i in indices {
                let p = static_cubes[i].position;
                min[0] = min[0].min(p.x - 0.5);
                min[1] = min[1].min(p.y - 0.5);
                min[2] = min[2].min(p.z - 0.5);
                max[0] = max[0].max(p.x + 0.5);
                max[1] = max[1].max(p.y + 0.5);
                max[2] = max[2].max(p.z + 0.5);
            }

            for &i in indices {
                let c = &static_cubes[i];
                instance_data.push(
                    Cube {
                        model: Matrix4::from_translation(c.position),
                        color: Vector4::new(c.color.x, c.color.y, c.color.z, 1.0),
                    }
                    .into_raw(),
                );
                cube_chunk_ids.push(chunk_idx as u32);
            }

            chunks_raw.push(ChunkRaw {
                aabb_min: min,
                _pad0: 0.0,
                aabb_max: max,
                start_idx,
                count,
                _pad1: [0; 3],
            });
        }

        let total_instance_count = instance_data.len() as u32;
        let num_chunks = chunks_raw.len() as u32;

        // ---------------------------------------------------------------
        // GPU buffers
        // ---------------------------------------------------------------

        let static_usage =
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;

        let all_instances_buffer = if instance_data.is_empty() {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Static Instances Buffer"),
                size: 4,
                usage: static_usage,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Static Instances Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: static_usage,
            })
        };

        let visible_size = (instance_data.len() * size_of::<CubeRaw>()).max(4) as u64;
        let visible_instances_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Visible Instances Buffer"),
            size: visible_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        // indirect: [index_count, instance_count, first_index, base_vertex, first_instance]
        let indirect_args = [INDICES.len() as u32, 0u32, 0u32, 0u32, 0u32];
        let indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Indirect Draw Buffer"),
            contents: bytemuck::cast_slice(&indirect_args),
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        let chunks_buffer = if chunks_raw.is_empty() {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Chunks Buffer"),
                size: 4,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunks Buffer"),
                contents: bytemuck::cast_slice(&chunks_raw),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

        let chunk_visible_size = (chunks_raw.len() * size_of::<u32>()).max(4) as u64;
        let chunk_visible_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Chunk Visible Buffer"),
            size: chunk_visible_size,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let cube_chunk_ids_buffer = if cube_chunk_ids.is_empty() {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Cube Chunk IDs Buffer"),
                size: 4,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cube Chunk IDs Buffer"),
                contents: bytemuck::cast_slice(&cube_chunk_ids),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

        // ---------------------------------------------------------------
        // Pipelines and bind groups
        // ---------------------------------------------------------------

        let shader = Shader::new(CUBE_SHADER, device, Some("Cube Shader"));
        let pipeline = Pipeline::new_v2(
            device,
            &[&camera.bind_group.layout],
            &[Vertex::desc(), Cube::desc()],
            &shader,
            target_format,
            Some(wgpu::TextureFormat::Depth32Float),
            wgpu::PrimitiveTopology::TriangleList,
            Some("Cubes Pipeline"),
        );

        let chunk_cull_pass = ChunkCullingPass::new(device, &camera.bind_group.layout);
        let chunk_culling_bind_group = chunk_cull_pass.create_bind_group(
            device,
            &chunks_buffer,
            &chunk_visible_buffer,
        );

        let cull_pass = CullingPass::new(device, &camera.bind_group.layout);
        let culling_bind_group = cull_pass.create_bind_group(
            device,
            &all_instances_buffer,
            &visible_instances_buffer,
            &indirect_buffer,
            &cube_chunk_ids_buffer,
            &chunk_visible_buffer,
        );

        let dynamic_scene = DynamicScene::new(device, dynamic_max_instances.max(1));

        Self {
            vertex_buffer,
            index_buffer,
            pipeline,
            all_instances_buffer,
            visible_instances_buffer,
            indirect_buffer,
            total_instance_count,
            chunk_cull_pass,
            chunk_culling_bind_group,
            num_chunks,
            chunks_buffer,
            chunk_visible_buffer,
            cull_pass,
            culling_bind_group,
            cube_chunk_ids_buffer,
            dynamic_scene,
            skybox: None,
            hdr,
            camera,
        }
    }

    // -------------------------------------------------------------------------
    // Skybox
    // -------------------------------------------------------------------------

    /// Load a skybox from raw equirectangular HDR bytes.
    ///
    /// `cubemap_resolution` is the edge length of each cubemap face.
    /// Replaces any previously loaded skybox.
    #[cfg(not(tarpaulin_include))]
    pub fn set_skybox_from_bytes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        cubemap_resolution: u32,
        rotation: cgmath::Matrix4<f32>,
    ) -> Result<()> {
        let loader = HdrLoader::new(device);
        let sky_texture = loader.from_equirectangular_bytes(
            device,
            queue,
            bytes,
            cubemap_resolution,
            Some("Skybox Texture"),
        )?;

        let rotation_raw: [[f32; 4]; 4] = rotation.into();

        let bind_group = BindGroupBuilder::new()
            .add_cube_texture(
                0,
                sky_texture.view().clone(),
                wgpu::ShaderStages::FRAGMENT,
                wgpu::TextureSampleType::Float { filterable: false },
            )
            .add_sampler(
                1,
                sky_texture.sampler().clone(),
                wgpu::SamplerBindingType::NonFiltering,
                wgpu::ShaderStages::FRAGMENT,
            )
            .add_uniform_buffer(
                device,
                2,
                bytemuck::bytes_of(&rotation_raw),
                wgpu::ShaderStages::FRAGMENT,
            )
            .build(device, Some("Skybox Bind Group"));

        let skybox_shader = Shader::new(SKYBOX_SHADER, device, Some("Skybox Shader"));
        let pipeline = Pipeline::new_skybox(
            device,
            &[&self.camera.bind_group.layout, &bind_group.layout],
            &skybox_shader,
            self.hdr.format(),
            wgpu::TextureFormat::Depth32Float,
            Some("Skybox Pipeline"),
        );

        self.skybox = Some(SkyboxData {
            pipeline,
            bind_group,
        });
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Dynamic models
    // -------------------------------------------------------------------------

    /// Add a dynamic model to the scene. Returns a stable ID used to move or remove it.
    pub fn add_dynamic(&mut self, model: DynamicModel) -> usize {
        self.dynamic_scene.add(model)
    }

    /// Remove a dynamic model by ID.
    pub fn remove_dynamic(&mut self, id: usize) {
        self.dynamic_scene.remove(id);
    }

    /// Get a shared reference to a dynamic model by ID.
    pub fn get_dynamic(&self, id: usize) -> Option<&DynamicModel> {
        self.dynamic_scene.get(id)
    }

    /// Get a mutable reference to a dynamic model by ID.
    ///
    /// The updated transform is uploaded to the GPU on the next [`render`] call.
    pub fn get_dynamic_mut(&mut self, id: usize) -> Option<&mut DynamicModel> {
        self.dynamic_scene.get_mut(id)
    }

    // -------------------------------------------------------------------------
    // Stats
    // -------------------------------------------------------------------------

    /// Total number of static cube instances.
    pub fn static_count(&self) -> u32 {
        self.total_instance_count
    }

    /// Total number of live dynamic cube instances across all dynamic models.
    pub fn dynamic_count(&self) -> u32 {
        self.dynamic_scene.live_count()
    }

    // -------------------------------------------------------------------------
    // Frame loop
    // -------------------------------------------------------------------------

    /// Resize the internal HDR texture to match the new window dimensions.
    ///
    /// Call this from [`Game::resize`] whenever the window is resized.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.hdr.resize(device, width, height);
    }

    /// Render the full scene to `swapchain_view`.
    ///
    /// Internally this:
    /// 1. Uploads changed dynamic transforms to the GPU.
    /// 2. Runs the GPU chunk-level frustum culling pass.
    /// 3. Runs the GPU per-cube frustum culling pass (skips cubes in culled chunks).
    /// 4. Opens an HDR render pass and draws static models, dynamic models,
    ///    and the skybox (if set).
    /// 5. Tonemaps the HDR result to `swapchain_view`.
    #[cfg(not(tarpaulin_include))]
    pub fn render(
        &self,
        ctx: &EngineContext,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let queue = &ctx.gfx.queue;

        // --- Upload dynamic transforms (before render pass) ---
        self.dynamic_scene.update_gpu(queue);

        if self.total_instance_count > 0 {
            // Reset the indirect instance_count to 0 before culling writes it.
            queue.write_buffer(&self.indirect_buffer, 4, bytemuck::bytes_of(&0u32));

            // Pass 1: chunk-level culling — writes chunk_visible[]
            self.chunk_cull_pass.cull(
                encoder,
                &self.camera.bind_group.bind_group,
                &self.chunk_culling_bind_group,
                self.num_chunks,
            );

            // Pass 2: per-cube culling — reads chunk_visible[], writes visible_instances[]
            self.cull_pass.cull(
                encoder,
                &self.camera.bind_group.bind_group,
                &self.culling_bind_group,
                self.total_instance_count,
            );
        }

        // --- Render pass on the HDR intermediate texture ---
        {
            let color_attachment = Gfx::color_attachments_from_view(self.hdr.view());
            let depth_attachment = wgpu::RenderPassDepthStencilAttachment {
                view: &ctx.gfx.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            };
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Scene Render Pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: Some(depth_attachment),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Cube pipeline — shared by static and dynamic
            render_pass.set_pipeline(&self.pipeline.pipeline);
            render_pass.set_bind_group(0, &self.camera.bind_group.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Static geometry — drawn via indirect after two-pass GPU culling
            if self.total_instance_count > 0 {
                render_pass.set_vertex_buffer(1, self.visible_instances_buffer.slice(..));
                render_pass.draw_indexed_indirect(&self.indirect_buffer, 0);
            }

            // Dynamic geometry
            if self.dynamic_scene.live_count() > 0 {
                render_pass.set_vertex_buffer(1, self.dynamic_scene.buffer().slice(..));
                render_pass.draw_indexed(
                    0..INDICES.len() as u32,
                    0,
                    0..self.dynamic_scene.live_count(),
                );
            }

            // Skybox (rendered last, at the far plane)
            if let Some(skybox) = &self.skybox {
                render_pass.set_pipeline(&skybox.pipeline.pipeline);
                render_pass.set_bind_group(0, &self.camera.bind_group.bind_group, &[]);
                render_pass.set_bind_group(1, &skybox.bind_group.bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }
        }

        // --- Tonemap HDR → swapchain ---
        self.hdr.process(encoder, view);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::{Camera, Projection};
    use cgmath::{Point3, Vector3};

    fn make_device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));
        if let Ok(adapter) = adapter {
            Some(
                pollster::block_on(
                    adapter.request_device(&wgpu::DeviceDescriptor::default()),
                )
                .unwrap(),
            )
        } else {
            None
        }
    }

    fn make_surface_config(w: u32, h: u32) -> wgpu::SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8Unorm,
            width: w,
            height: h,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

    fn white_cube(x: f32, y: f32, z: f32) -> ModelCube {
        ModelCube {
            position: Vector3::new(x, y, z),
            color: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    fn build_scene(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        static_cubes: &[ModelCube],
        dynamic_max: usize,
    ) -> Scene {
        use crate::hdr::{HdrPipeline, TonemappingMode};

        let hdr = HdrPipeline::new(device, surface_config, TonemappingMode::Sdr, 1000.0);
        let target_format = hdr.format();

        let camera = Camera::new(
            device,
            Point3::new(0.0, 5.0, 10.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
            surface_config.width as f32 / surface_config.height as f32,
            Projection::Perspective { fovy: 60.0 },
            0.1,
            1000.0,
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut chunk_map: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (i, cube) in static_cubes.iter().enumerate() {
            let cx = (cube.position.x / CHUNK_SIZE).floor() as i32;
            let cz = (cube.position.z / CHUNK_SIZE).floor() as i32;
            chunk_map.entry((cx, cz)).or_default().push(i);
        }
        let mut chunk_keys: Vec<(i32, i32)> = chunk_map.keys().copied().collect();
        chunk_keys.sort_unstable();

        let mut instance_data: Vec<CubeRaw> = Vec::with_capacity(static_cubes.len());
        let mut cube_chunk_ids: Vec<u32> = Vec::with_capacity(static_cubes.len());
        let mut chunks_raw: Vec<ChunkRaw> = Vec::with_capacity(chunk_keys.len());

        for (chunk_idx, key) in chunk_keys.iter().enumerate() {
            let indices = &chunk_map[key];
            let start_idx = instance_data.len() as u32;
            let count = indices.len() as u32;
            let mut min = [f32::MAX; 3];
            let mut max = [f32::MIN; 3];
            for &i in indices {
                let p = static_cubes[i].position;
                min[0] = min[0].min(p.x - 0.5);
                min[1] = min[1].min(p.y - 0.5);
                min[2] = min[2].min(p.z - 0.5);
                max[0] = max[0].max(p.x + 0.5);
                max[1] = max[1].max(p.y + 0.5);
                max[2] = max[2].max(p.z + 0.5);
            }
            for &i in indices {
                let c = &static_cubes[i];
                instance_data.push(
                    Cube {
                        model: Matrix4::from_translation(c.position),
                        color: Vector4::new(c.color.x, c.color.y, c.color.z, 1.0),
                    }
                    .into_raw(),
                );
                cube_chunk_ids.push(chunk_idx as u32);
            }
            chunks_raw.push(ChunkRaw {
                aabb_min: min,
                _pad0: 0.0,
                aabb_max: max,
                start_idx,
                count,
                _pad1: [0; 3],
            });
        }

        let total_instance_count = instance_data.len() as u32;
        let num_chunks = chunks_raw.len() as u32;

        let static_usage = wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST;

        let all_instances_buffer = if instance_data.is_empty() {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Static Instances Buffer"),
                size: 4,
                usage: static_usage,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Static Instances Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: static_usage,
            })
        };

        let visible_size = (instance_data.len() * size_of::<CubeRaw>()).max(4) as u64;
        let visible_instances_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Visible Instances Buffer"),
            size: visible_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let indirect_args = [INDICES.len() as u32, 0u32, 0u32, 0u32, 0u32];
        let indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Indirect Draw Buffer"),
            contents: bytemuck::cast_slice(&indirect_args),
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        let chunks_buffer = if chunks_raw.is_empty() {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Chunks Buffer"),
                size: 4,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunks Buffer"),
                contents: bytemuck::cast_slice(&chunks_raw),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

        let chunk_visible_size = (chunks_raw.len() * size_of::<u32>()).max(4) as u64;
        let chunk_visible_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Chunk Visible Buffer"),
            size: chunk_visible_size,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let cube_chunk_ids_buffer = if cube_chunk_ids.is_empty() {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Cube Chunk IDs Buffer"),
                size: 4,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cube Chunk IDs Buffer"),
                contents: bytemuck::cast_slice(&cube_chunk_ids),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

        let shader = Shader::new(CUBE_SHADER, device, Some("Cube Shader"));
        let pipeline = Pipeline::new_v2(
            device,
            &[&camera.bind_group.layout],
            &[Vertex::desc(), Cube::desc()],
            &shader,
            target_format,
            Some(wgpu::TextureFormat::Depth32Float),
            wgpu::PrimitiveTopology::TriangleList,
            Some("Cubes Pipeline"),
        );

        let chunk_cull_pass = ChunkCullingPass::new(device, &camera.bind_group.layout);
        let chunk_culling_bind_group = chunk_cull_pass.create_bind_group(
            device,
            &chunks_buffer,
            &chunk_visible_buffer,
        );

        let cull_pass = CullingPass::new(device, &camera.bind_group.layout);
        let culling_bind_group = cull_pass.create_bind_group(
            device,
            &all_instances_buffer,
            &visible_instances_buffer,
            &indirect_buffer,
            &cube_chunk_ids_buffer,
            &chunk_visible_buffer,
        );

        let dynamic_scene = DynamicScene::new(device, dynamic_max.max(1));

        Scene {
            vertex_buffer,
            index_buffer,
            pipeline,
            all_instances_buffer,
            visible_instances_buffer,
            indirect_buffer,
            total_instance_count,
            chunk_cull_pass,
            chunk_culling_bind_group,
            num_chunks,
            chunks_buffer,
            chunk_visible_buffer,
            cull_pass,
            culling_bind_group,
            cube_chunk_ids_buffer,
            dynamic_scene,
            skybox: None,
            hdr,
            camera,
        }
    }

    #[test]
    fn test_scene_new_empty_statics() {
        let Some((device, _)) = make_device() else { return; };
        let cfg = make_surface_config(800, 600);
        let scene = build_scene(&device, &cfg, &[], 10);
        assert_eq!(scene.static_count(), 0);
        assert_eq!(scene.dynamic_count(), 0);
    }

    #[test]
    fn test_scene_new_with_statics() {
        let Some((device, _)) = make_device() else { return; };
        let cfg = make_surface_config(800, 600);
        let cubes = vec![
            white_cube(0.0, 0.0, 0.0),
            white_cube(1.0, 0.0, 0.0),
            white_cube(33.0, 0.0, 0.0),
        ];
        let scene = build_scene(&device, &cfg, &cubes, 10);
        assert_eq!(scene.static_count(), 3);
    }

    #[test]
    fn test_static_count() {
        let Some((device, _)) = make_device() else { return; };
        let cfg = make_surface_config(800, 600);
        let cubes = vec![white_cube(0.0, 0.0, 0.0), white_cube(1.0, 0.0, 0.0)];
        let scene = build_scene(&device, &cfg, &cubes, 10);
        assert_eq!(scene.static_count(), 2);
    }

    #[test]
    fn test_dynamic_count_starts_at_zero() {
        let Some((device, _)) = make_device() else { return; };
        let cfg = make_surface_config(800, 600);
        let scene = build_scene(&device, &cfg, &[], 10);
        assert_eq!(scene.dynamic_count(), 0);
    }

    #[test]
    fn test_add_get_dynamic() {
        let Some((device, _)) = make_device() else { return; };
        let cfg = make_surface_config(800, 600);
        let mut scene = build_scene(&device, &cfg, &[], 10);
        let model = DynamicModel::from_cubes(vec![white_cube(1.0, 0.0, 0.0)]);
        let id = scene.add_dynamic(model);
        assert!(scene.get_dynamic(id).is_some());
        assert!(scene.get_dynamic(999).is_none());
    }

    #[test]
    fn test_get_dynamic_mut() {
        let Some((device, _)) = make_device() else { return; };
        let cfg = make_surface_config(800, 600);
        let mut scene = build_scene(&device, &cfg, &[], 10);
        let model = DynamicModel::from_cubes(vec![white_cube(0.0, 0.0, 0.0)]);
        let id = scene.add_dynamic(model);
        {
            let m = scene.get_dynamic_mut(id).unwrap();
            m.position = Vector3::new(5.0, 0.0, 0.0);
        }
        assert_eq!(
            scene.get_dynamic(id).unwrap().position,
            Vector3::new(5.0, 0.0, 0.0)
        );
        assert!(scene.get_dynamic_mut(999).is_none());
    }

    #[test]
    fn test_remove_dynamic() {
        let Some((device, _)) = make_device() else { return; };
        let cfg = make_surface_config(800, 600);
        let mut scene = build_scene(&device, &cfg, &[], 10);
        let model = DynamicModel::from_cubes(vec![white_cube(0.0, 0.0, 0.0)]);
        let id = scene.add_dynamic(model);
        scene.remove_dynamic(id);
        assert!(scene.get_dynamic(id).is_none());
        scene.remove_dynamic(id);
        scene.remove_dynamic(999);
    }

    #[test]
    fn test_scene_resize() {
        let Some((device, _)) = make_device() else { return; };
        let cfg = make_surface_config(800, 600);
        let mut scene = build_scene(&device, &cfg, &[], 10);
        // Resize must not panic and the HDR pipeline must still be usable afterwards
        scene.resize(&device, 1280, 720);
        assert_eq!(scene.hdr.format(), wgpu::TextureFormat::Rgba16Float);
        let _view: &wgpu::TextureView = scene.hdr.view();
    }

    #[test]
    fn test_scene_multi_chunk() {
        let Some((device, _)) = make_device() else { return; };
        let cfg = make_surface_config(800, 600);
        let cubes = vec![
            white_cube(0.0, 0.0, 0.0),
            white_cube(33.0, 0.0, 0.0),
            white_cube(66.0, 0.0, 0.0),
        ];
        let scene = build_scene(&device, &cfg, &cubes, 10);
        assert_eq!(scene.static_count(), 3);
        assert_eq!(scene.num_chunks, 3);
    }
}
