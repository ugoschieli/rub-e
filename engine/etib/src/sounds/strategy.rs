use cgmath::Vector3;

pub struct EmitterState {
    pub position: Vector3<f32>,
    pub volume: f32,
}

pub trait SoundStrategy: Send + Sync {
    fn compute_emitters(
        &self,
        cubes: &[Vector3<f32>],
        camera_pos: Vector3<f32>,
    ) -> Vec<EmitterState>;
}

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
