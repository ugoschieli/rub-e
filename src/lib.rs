#![warn(missing_docs)]

//! The ETIB Game engine crate.

mod buffer;
mod camera;
mod cube;
mod game;
mod gfx;
mod pipeline;
mod shader;
mod uniform;
mod vertex;

pub use crate::buffer::*;
pub use crate::camera::*;
pub use crate::cube::*;
pub use crate::game::*;
pub use crate::gfx::*;
pub use crate::pipeline::*;
pub use crate::shader::*;
pub use crate::uniform::*;
pub use crate::vertex::*;
