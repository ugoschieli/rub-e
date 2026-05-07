use cgmath::Vector3;
use kira::{
    AudioManager, DefaultBackend, Tween,
    listener::ListenerHandle,
    sound::static_sound::StaticSoundData,
    track::{SpatialTrackBuilder, SpatialTrackHandle},
};

use crate::sounds::strategy::SoundStrategy;

fn to_mint(v: Vector3<f32>) -> mint::Vector3<f32> {
    mint::Vector3 {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

/// A group of spatial emitters playing the same sound.
///
/// The group owns a single spatial track and updates its position/volume
/// every frame based on a [`SoundStrategy`].
pub struct SoundGroup {
    cubes: Vec<Vector3<f32>>,
    strategy: Box<dyn SoundStrategy>,
    track: SpatialTrackHandle,
}

impl SoundGroup {
    /// Create a new group and start looping the given sound.
    pub fn new(
        manager: &mut AudioManager<DefaultBackend>,
        listener: &ListenerHandle,
        cubes: Vec<Vector3<f32>>,
        sound_data: StaticSoundData,
        strategy: Box<dyn SoundStrategy>,
    ) -> Self {
        let pos = mint::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        let mut track = manager
            .add_spatial_sub_track(listener.id(), pos, SpatialTrackBuilder::new())
            .expect("Failed to create spatial track");

        track
            .play(sound_data.loop_region(..))
            .expect("Failed to play sound");

        Self {
            cubes,
            strategy,
            track,
        }
    }

    /// Replace the cube positions used by the strategy.
    pub fn update_cubes(&mut self, cubes: Vec<Vector3<f32>>) {
        self.cubes = cubes;
    }

    /// Update the group's emitter state for this frame.
    pub fn update(&mut self, camera_pos: Vector3<f32>) {
        let emitters = self.strategy.compute_emitters(&self.cubes, camera_pos);

        if let Some(state) = emitters.first() {
            self.track
                .set_position(to_mint(state.position), Tween::default());
            self.track.set_volume(state.volume, Tween::default());
        }
    }
}
