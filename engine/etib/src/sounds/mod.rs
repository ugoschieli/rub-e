//! 3D spatial audio utilities built on top of the `kira` audio engine.
//!
//! This module provides a high-level [`SoundManager`] that updates a listener
//! from the engine camera, and one or more sound "groups" that can implement
//! different emitter placement strategies.

/// Sound group implementation (tracks + emitter updates).
pub mod group;
/// High-level audio manager (listener + groups).
pub mod manager;
/// Emitter strategies (how to place one or more emitters from cube positions).
pub mod strategy;

/// High-level manager for spatial sounds.
pub use manager::SoundManager;
/// Re-exported so callers can preload sounds without depending on `kira` directly.
pub use kira::sound::static_sound::StaticSoundData;
