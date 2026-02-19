#![warn(missing_docs)]

//! The ETIB Game engine crate.

/// Module containing the camera implementation
pub mod camera;
/// Module containing the cube data (Vertex + Instance) and model loading
pub mod cube;
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
/// Module containing the time management (delta time)
pub mod time;
mod vertex;

pub use crate::game::*;
pub use crate::gfx::*;
pub use crate::scene::Scene;
pub use crate::vertex::*;
