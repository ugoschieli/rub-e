#![warn(missing_docs)]

//! The ETIB Game engine crate.

/// Module containing the camera implementation
pub mod camera;
/// Core module containing wrapper for wgpu and time management
pub mod core;
/// Module containing the cube data (Vertex + Instance) and model loading
pub mod cube;
mod game;
mod gbuffer;
mod gfx;
/// Utility module
pub mod utils;
// mod raytracing;
/// Module containing the engine configuration system
pub mod config;
/// Module containing the time management (delta time)
pub mod time;
mod vertex;

pub use crate::game::*;
pub use crate::gfx::*;
pub use crate::vertex::*;
