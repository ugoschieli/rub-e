use cgmath::InnerSpace;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend, Tween};

use crate::camera::Camera;
use crate::sounds::{group::SoundGroup, strategy::SoundStrategy};

fn to_mint_vec3(x: f32, y: f32, z: f32) -> mint::Vector3<f32> {
    mint::Vector3 { x, y, z }
}

fn to_mint_quat(x: f32, y: f32, z: f32, w: f32) -> mint::Quaternion<f32> {
    mint::Quaternion {
        v: mint::Vector3 { x, y, z },
        s: w,
    }
}

pub struct SoundManager {
    audio_manager: AudioManager<DefaultBackend>,
    listener: kira::listener::ListenerHandle,
    groups: Vec<SoundGroup>,
}

impl SoundManager {
    pub fn new() -> Self {
        let mut audio_manager = AudioManager::new(AudioManagerSettings::default())
            .expect("Failed to create audio manager");

        let listener = audio_manager
            .add_listener(
                to_mint_vec3(0.0, 0.0, 0.0),
                to_mint_quat(0.0, 0.0, 0.0, 1.0),
            )
            .expect("Failed to create listener");

        Self {
            audio_manager,
            listener,
            groups: Vec::new(),
        }
    }

    pub fn add_group(
        &mut self,
        cubes: Vec<cgmath::Vector3<f32>>,
        sound_path: &str,
        strategy: Box<dyn SoundStrategy>,
    ) {
        let sound_data = kira::sound::static_sound::StaticSoundData::from_file(sound_path)
            .expect("Failed to load sound file");

        let group = SoundGroup::new(
            &mut self.audio_manager,
            &self.listener,
            cubes,
            sound_data,
            strategy,
        );

        self.groups.push(group);
    }

    pub fn update(&mut self, camera: &Camera) {
        // Position du listener
        self.listener.set_position(
            to_mint_vec3(camera.eye.x, camera.eye.y, camera.eye.z),
            Tween::default(),
        );

        // Orientation : on calcule un quaternion depuis la direction forward
        let forward = (camera.target - camera.eye).normalize();
        let up = camera.up;

        // Gram-Schmidt pour construire une base orthonormée
        let f = cgmath::Vector3::new(forward.x, forward.y, forward.z).normalize();
        let r = f.cross(up).normalize();
        let u = r.cross(f);

        // Matrice de rotation 3x3 → quaternion
        let trace = r.x + u.y + (-f.z); // -f car on regarde vers -Z
        let quat = if trace > 0.0 {
            let s = 0.5 / (trace + 1.0_f32).sqrt();
            mint::Quaternion {
                v: mint::Vector3 {
                    x: (u.z - (-f.y)) * s,
                    y: ((-f.x) - r.z) * s,
                    z: (r.y - u.x) * s,
                },
                s: 0.25 / s,
            }
        } else {
            // Fallback identité
            to_mint_quat(0.0, 0.0, 0.0, 1.0)
        };

        self.listener.set_orientation(quat, Tween::default());

        // Mise à jour des groupes
        let camera_pos = cgmath::Vector3::new(camera.eye.x, camera.eye.y, camera.eye.z);
        for group in &mut self.groups {
            group.update(camera_pos);
        }
    }
}

impl Default for SoundManager {
    fn default() -> Self {
        Self::new()
    }
}
