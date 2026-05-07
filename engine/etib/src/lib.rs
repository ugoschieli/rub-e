#![warn(missing_docs)]

//! The ETIB Game engine crate.

/// Module containing the camera implementation
pub mod camera;
/// Module containing the cube data (Vertex + Instance) and model loading
pub mod cube;

#[cfg(not(tarpaulin_include))]
mod game;
mod gfx;
/// High-level scene renderer (static + dynamic models, culling, draw calls)
pub mod scene;
/// Utility module
pub mod utils;
// mod raytracing;
/// Module containing the engine configuration system
pub mod config;
/// An HDR pipeline implementation
pub mod hdr;
/// Module providing keyboard input state tracking
pub mod input;
/// 3D spatial audio helpers (sound manager + strategies).
pub mod sounds;
/// Module containing the time management (delta time)
pub mod time;
mod vertex;

/// Module containing the experimental raytracing renderer
#[cfg(not(tarpaulin_include))]
pub mod raytracing;
/// Simple rigid-body physics: gravity integration and floor collision
pub mod physics;

pub use crate::game::*;
pub use crate::gfx::*;
pub use crate::scene::Scene;
pub use crate::vertex::*;
pub use egui;
