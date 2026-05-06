//! Dynamic model management for runtime-created, independently moving models.
//!
//! A dynamic model is a group of cubes loaded from a `.model` file. The group
//! can be repositioned, rotated, and scaled at runtime as a unit — each cube's
//! position is defined relative to the model's local origin, and the group
//! transform is applied on top every frame.

use std::cell::Cell;

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
    live_instance_count: Cell<u32>,
    dirty: Cell<bool>,
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
            live_instance_count: Cell::new(0),
            dirty: Cell::new(false),
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
            self.dirty.set(true);
            return pos;
        }
        let id = self.models.len();
        self.models.push(Some(model));
        self.dirty.set(true);
        id
    }

    /// Remove a model by ID. Its cube slots are freed.
    pub fn remove(&mut self, id: usize) {
        if id < self.models.len() && self.models[id].is_some() {
            self.models[id] = None;
            self.dirty.set(true);
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
        self.dirty.set(true);
        Some(slot)
    }

    /// Upload changed transforms to the GPU. Call **once per frame** before rendering.
    ///
    /// All live model cubes are packed contiguously into the buffer with their
    /// group transforms applied. Does nothing if nothing changed since the last call.
    pub fn update_gpu(&self, queue: &wgpu::Queue) {
        if !self.dirty.get() {
            return;
        }
        let raw: Vec<CubeRaw> = self
            .models
            .iter()
            .filter_map(|m| m.as_ref())
            .flat_map(|m| m.to_raw_instances())
            .collect();
        self.live_instance_count.set(raw.len() as u32);
        if !raw.is_empty() {
            queue.write_buffer(&self.gpu_buffer, 0, bytemuck::cast_slice(&raw));
        }
        self.dirty.set(false);
    }

    /// Total number of live cube instances across all models.
    /// Use as the instance count in draw calls after [`update_gpu`].
    pub fn live_count(&self) -> u32 {
        self.live_instance_count.get()
    }

    /// The GPU instance buffer containing all live model instances.
    ///
    /// Bind as vertex slot 1 — the layout is identical to the static instance buffer.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.gpu_buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));
        if let Ok(adapter) = adapter {
            Some(
                pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                    .unwrap(),
            )
        } else {
            None
        }
    }

    #[test]
    fn test_dynamic_model_load() {
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "1.0 2.0 3.0 1.0 0.0 0.0").unwrap();
        writeln!(file, "4.0 5.0 6.0").unwrap();

        let model = DynamicModel::load(file.path().to_str().unwrap()).unwrap();
        assert_eq!(model.cube_count(), 2);
        assert_eq!(model.position, Vector3::zero());
        assert_eq!(model.scale, 1.0);
    }

    #[test]
    fn test_dynamic_scene_new_and_buffer() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let scene = DynamicScene::new(&device, 100);
        assert_eq!(scene.live_count(), 0);
        let _buf: &wgpu::Buffer = scene.buffer();
    }

    #[test]
    fn test_dynamic_scene_add_and_get() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let mut scene = DynamicScene::new(&device, 100);

        let model = DynamicModel::from_cubes(vec![ModelCube {
            position: Vector3::new(1.0, 0.0, 0.0),
            color: Vector3::new(1.0, 1.0, 1.0),
        }]);

        let id = scene.add(model);
        assert_eq!(id, 0);
        assert!(scene.get(id).is_some());
        assert!(scene.get(999).is_none());
    }

    #[test]
    fn test_dynamic_scene_get_mut() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let mut scene = DynamicScene::new(&device, 100);
        let id = scene.add(DynamicModel::from_cubes(vec![ModelCube {
            position: Vector3::new(0.0, 0.0, 0.0),
            color: Vector3::new(1.0, 1.0, 1.0),
        }]));

        let m = scene.get_mut(id).unwrap();
        m.position = Vector3::new(5.0, 0.0, 0.0);
        assert_eq!(scene.get(id).unwrap().position, Vector3::new(5.0, 0.0, 0.0));
        assert!(scene.get_mut(999).is_none());
    }

    #[test]
    fn test_dynamic_scene_remove() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let mut scene = DynamicScene::new(&device, 100);
        let id = scene.add(DynamicModel::from_cubes(vec![ModelCube {
            position: Vector3::zero(),
            color: Vector3::new(1.0, 1.0, 1.0),
        }]));
        scene.remove(id);
        assert!(scene.get(id).is_none());
        scene.remove(id);   // double-remove is safe
        scene.remove(999);  // out-of-bounds remove is safe
    }

    #[test]
    fn test_dynamic_scene_update_gpu_live_count() {
        let Some((device, queue)) = make_device() else {
            return;
        };
        let mut scene = DynamicScene::new(&device, 100);
        assert_eq!(scene.live_count(), 0);

        scene.add(DynamicModel::from_cubes(vec![
            ModelCube { position: Vector3::zero(), color: Vector3::new(1.0, 1.0, 1.0) },
            ModelCube { position: Vector3::new(1.0, 0.0, 0.0), color: Vector3::new(0.0, 1.0, 0.0) },
        ]));

        assert_eq!(scene.live_count(), 0); // not uploaded yet
        scene.update_gpu(&queue);
        assert_eq!(scene.live_count(), 2);

        scene.update_gpu(&queue); // dirty=false, no-op
        assert_eq!(scene.live_count(), 2);
    }

    #[test]
    fn test_dynamic_scene_reuses_freed_slots() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let make_model = || {
            DynamicModel::from_cubes(vec![ModelCube {
                position: Vector3::zero(),
                color: Vector3::new(1.0, 1.0, 1.0),
            }])
        };
        let mut scene = DynamicScene::new(&device, 100);
        let id0 = scene.add(make_model());
        let id1 = scene.add(make_model());
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        scene.remove(id0);
        let id_reused = scene.add(make_model());
        assert_eq!(id_reused, 0); // freed slot should be reused
    }

    #[test]
    fn test_dynamic_scene_exceeds_capacity_panics() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut scene = DynamicScene::new(&device, 1);
            let model = DynamicModel::from_cubes(vec![
                ModelCube { position: Vector3::zero(), color: Vector3::new(1.0, 1.0, 1.0) },
                ModelCube { position: Vector3::new(1.0, 0.0, 0.0), color: Vector3::new(1.0, 1.0, 1.0) },
            ]);
            scene.add(model);
        }));
        assert!(result.is_err(), "exceeding max_instances should panic");
    }

    #[test]
    fn test_dynamic_model() {
        let cubes = vec![
            ModelCube {
                position: Vector3::new(1.0, 2.0, 3.0),
                color: Vector3::new(0.5, 0.5, 0.5),
            },
        ];
        
        let mut model = DynamicModel::from_cubes(cubes);
        assert_eq!(model.cube_count(), 1);
        
        model.position = Vector3::new(10.0, 0.0, 0.0);
        
        {
            let mut raw = model.to_raw_instances();
            let instance = raw.next().unwrap();
            let bytes = bytemuck::bytes_of(&instance);
            let float_array: &[f32] = bytemuck::cast_slice(bytes);
            assert_eq!(float_array[12], 11.0);
            assert_eq!(float_array[13], 2.0);
        }
        
        model.cubes_mut()[0].position.x = 5.0;
        let mut raw2 = model.to_raw_instances();
        let instance2 = raw2.next().unwrap();
        let bytes2 = bytemuck::bytes_of(&instance2);
        let float_array2: &[f32] = bytemuck::cast_slice(bytes2);
        assert_eq!(float_array2[12], 15.0);
    }

    #[test]
    fn test_dynamic_model_from_cubes_empty() {
        let model = DynamicModel::from_cubes(vec![]);
        assert_eq!(model.cube_count(), 0);
        let raw: Vec<_> = model.to_raw_instances().collect();
        assert!(raw.is_empty());
    }

    #[test]
    fn test_dynamic_model_scale() {
        let cubes = vec![ModelCube {
            position: Vector3::new(1.0, 0.0, 0.0),
            color: Vector3::new(1.0, 1.0, 1.0),
        }];
        let mut model = DynamicModel::from_cubes(cubes);
        model.scale = 2.0;

        let instance = model.to_raw_instances().next().unwrap();
        let floats: &[f32] = bytemuck::cast_slice(bytemuck::bytes_of(&instance));
        // Scale is applied to the local translation: (2 * 1, 0, 0)
        assert_eq!(floats[12], 2.0); // tx
        assert_eq!(floats[13], 0.0); // ty
        assert_eq!(floats[14], 0.0); // tz
    }

    #[test]
    fn test_dynamic_model_rotation() {
        use cgmath::Rotation3;
        let cubes = vec![ModelCube {
            position: Vector3::new(1.0, 0.0, 0.0),
            color: Vector3::new(1.0, 1.0, 1.0),
        }];
        let mut model = DynamicModel::from_cubes(cubes);
        // 90° around Y: (1,0,0) → (0,0,-1)
        model.rotation = Quaternion::from_angle_y(cgmath::Deg(90.0));

        let instance = model.to_raw_instances().next().unwrap();
        let floats: &[f32] = bytemuck::cast_slice(bytemuck::bytes_of(&instance));
        assert!(floats[12].abs() < 1e-5, "x should be ~0");
        assert!(floats[13].abs() < 1e-5, "y should be ~0");
        assert!((floats[14] + 1.0).abs() < 1e-5, "z should be ~-1");
    }

    #[test]
    fn test_dynamic_model_cube_count() {
        let cubes = vec![
            ModelCube { position: Vector3::new(0.0, 0.0, 0.0), color: Vector3::new(1.0, 0.0, 0.0) },
            ModelCube { position: Vector3::new(1.0, 0.0, 0.0), color: Vector3::new(0.0, 1.0, 0.0) },
            ModelCube { position: Vector3::new(2.0, 0.0, 0.0), color: Vector3::new(0.0, 0.0, 1.0) },
        ];
        let model = DynamicModel::from_cubes(cubes);
        assert_eq!(model.cube_count(), 3);
    }
}
