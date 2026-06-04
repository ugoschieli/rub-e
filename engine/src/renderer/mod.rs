use std::fmt::Debug;

use winit::dpi::PhysicalSize;

use crate::{camera::Camera, chunk::World, gfx::Gfx};

pub mod old_renderer;
pub mod static_renderer;

pub trait Renderer: Debug {
    fn render(&mut self, gfx: &mut Gfx, world: &World, camera: &Camera, size: PhysicalSize<u32>);
}
