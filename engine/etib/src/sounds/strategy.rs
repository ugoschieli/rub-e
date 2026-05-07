use cgmath::Vector3;

/// A single spatial emitter state for one frame.
pub struct EmitterState {
    /// World-space position of the emitter.
    pub position: Vector3<f32>,
    /// Linear volume multiplier.
    pub volume: f32,
}

/// Strategy interface for converting cube positions into one or more emitters.
pub trait SoundStrategy: Send + Sync {
    /// Compute emitters from the set of cube positions and the current camera position.
    fn compute_emitters(
        &self,
        cubes: &[Vector3<f32>],
        camera_pos: Vector3<f32>,
    ) -> Vec<EmitterState>;
}

/// Simplest strategy: use the first cube as the single emitter.
pub struct SingleCubeStrategy;

impl SoundStrategy for SingleCubeStrategy {
    fn compute_emitters(
        &self,
        cubes: &[Vector3<f32>],
        _camera_pos: Vector3<f32>,
    ) -> Vec<EmitterState> {
        cubes
            .first()
            .map(|&position| {
                vec![EmitterState {
                    position,
                    volume: 1.0,
                }]
            })
            .unwrap_or_default()
    }
}
