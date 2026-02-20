//! Dynamic model management for runtime-created, independently moving models.
//!
//! A dynamic model is a group of cubes loaded from a `.model` file. The group
//! can be repositioned, rotated, and scaled at runtime as a unit — each cube's
//! position is defined relative to the model's local origin, and the group
//! transform is applied on top every frame.

use anyhow::Result;
use cgmath::{Matrix4, Quaternion, Vector3, Vector4, Zero};

use super::cube::{Cube, CubeRaw};
use super::model::{ModelCube, load_model};

/// A multi-cube model loaded from a file that can move independently at runtime.
///
/// Each cube's position is stored in model-local space. The group transform
/// (`position`, `rotation`, `scale`) is applied on top when uploading to the GPU,
/// so moving the model moves all its cubes together as a rigid body.
pub struct DynamicModel {
    cubes: Vec<ModelCube>,
    /// World-space position of the model origin.
    pub position: Vector3<f32>,
    /// Rotation of the whole model around its origin. Defaults to identity.
    pub rotation: Quaternion<f32>,
    /// Uniform scale applied to the whole model. Defaults to `1.0`.
    pub scale: f32,
}

impl DynamicModel {
    /// Load a dynamic model from a `.model` file.
    ///
    /// The model starts at the world origin with identity rotation and scale 1.
    /// Set `position`, `rotation`, and `scale` to place it in the scene.
    pub fn load(path: &str) -> Result<Self> {
        let cubes = load_model(path)?;
        Ok(Self::from_cubes(cubes))
    }

    /// Create a dynamic model from an already-loaded list of cubes.
    pub fn from_cubes(cubes: Vec<ModelCube>) -> Self {
        Self {
            cubes,
            position: Vector3::zero(),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: 1.0,
        }
    }

    /// Access the cubes mutably, for instance to change their color.
    pub fn cubes_mut(&mut self) -> &mut [ModelCube] {
        &mut self.cubes
    }

    /// Number of cubes in this model, which equals the number of GPU instances it occupies.
    pub fn cube_count(&self) -> usize {
        self.cubes.len()
    }

    fn to_raw_instances(&self) -> impl Iterator<Item = CubeRaw> + '_ {
        let group = Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_scale(self.scale);

        self.cubes.iter().map(move |cube| {
            // Combine the group transform with each cube's local translation.
            let model = group * Matrix4::from_translation(cube.position);
            Cube {
                model,
                color: Vector4::new(cube.color.x, cube.color.y, cube.color.z, 1.0),
            }
            .into_raw()
        })
    }
}

/// Manages a collection of dynamic models and their shared GPU instance buffer.
///
/// Each dynamic model is a full multi-cube structure loaded from a `.model` file.
/// All models in the scene share one flat GPU buffer; on every frame their
/// transformed cube instances are packed contiguously into it.
///
/// # Usage
///
/// 1. **Init**: `let mut scene = DynamicScene::new(&device, max_total_cubes);`
/// 2. **Add**: `let id = scene.add(DynamicModel::load("models/ship.model")?);`
/// 3. **Move** (each frame): `scene.get_mut(id).unwrap().position = ...;`
/// 4. **Upload** (once per frame, before rendering): `scene.update_gpu(&queue);`
/// 5. **Draw**: bind `scene.buffer()` as vertex slot 1 and draw `scene.live_count()` instances.
///
/// Dynamic models share the same render pipeline and shader as static ones —
/// no extra GPU resources are needed.
pub struct DynamicScene {
    models: Vec<Option<DynamicModel>>,
    gpu_buffer: wgpu::Buffer,
    max_instances: usize,
    live_instance_count: u32,
    dirty: bool,
}

impl DynamicScene {
    /// Create a new dynamic scene.
    ///
    /// `max_instances` is the upper bound on the **total number of cubes** across
    /// all live models combined. The GPU buffer is pre-allocated accordingly.
    pub fn new(device: &wgpu::Device, max_instances: usize) -> Self {
        let buffer_size = ((max_instances * size_of::<CubeRaw>()) as u64).max(64);
        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dynamic Instances Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            models: Vec::new(),
            gpu_buffer,
            max_instances,
            live_instance_count: 0,
            dirty: false,
        }
    }

    /// Add a model to the scene. Returns a stable ID for later access.
    ///
    /// Freed slots from [`remove`] are reused before growing the list.
    ///
    /// # Panics
    /// Panics if the total cube count across all live models would exceed `max_instances`.
    pub fn add(&mut self, model: DynamicModel) -> usize {
        let used: usize = self
            .models
            .iter()
            .filter_map(|m| m.as_ref())
            .map(|m| m.cube_count())
            .sum();
        assert!(
            used + model.cube_count() <= self.max_instances,
            "DynamicScene: max_instances ({}) exceeded",
            self.max_instances
        );

        if let Some(pos) = self.models.iter().position(|m| m.is_none()) {
            self.models[pos] = Some(model);
            self.dirty = true;
            return pos;
        }
        let id = self.models.len();
        self.models.push(Some(model));
        self.dirty = true;
        id
    }

    /// Remove a model by ID. Its cube slots are freed.
    pub fn remove(&mut self, id: usize) {
        if id < self.models.len() && self.models[id].is_some() {
            self.models[id] = None;
            self.dirty = true;
        }
    }

    /// Get a shared reference to a model by ID.
    pub fn get(&self, id: usize) -> Option<&DynamicModel> {
        self.models.get(id)?.as_ref()
    }

    /// Get a mutable reference to a model by ID.
    ///
    /// Marks the scene dirty so changes are uploaded on the next [`update_gpu`] call.
    pub fn get_mut(&mut self, id: usize) -> Option<&mut DynamicModel> {
        let slot = self.models.get_mut(id)?.as_mut()?;
        self.dirty = true;
        Some(slot)
    }

    /// Upload changed transforms to the GPU. Call **once per frame** before rendering.
    ///
    /// All live model cubes are packed contiguously into the buffer with their
    /// group transforms applied. Does nothing if nothing changed since the last call.
    pub fn update_gpu(&mut self, queue: &wgpu::Queue) {
        if !self.dirty {
            return;
        }
        let raw: Vec<CubeRaw> = self
            .models
            .iter()
            .filter_map(|m| m.as_ref())
            .flat_map(|m| m.to_raw_instances())
            .collect();
        self.live_instance_count = raw.len() as u32;
        if !raw.is_empty() {
            queue.write_buffer(&self.gpu_buffer, 0, bytemuck::cast_slice(&raw));
        }
        self.dirty = false;
    }

    /// Total number of live cube instances across all models.
    /// Use as the instance count in draw calls after [`update_gpu`].
    pub fn live_count(&self) -> u32 {
        self.live_instance_count
    }

    /// The GPU instance buffer containing all live model instances.
    ///
    /// Bind as vertex slot 1 — the layout is identical to the static instance buffer.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.gpu_buffer
    }
}
