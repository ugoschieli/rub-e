use std::fmt::Debug;

use winit::dpi::PhysicalSize;

use crate::{camera::Camera, gfx::Gfx};

pub mod static_renderer;
pub mod dynamic_renderer;

pub trait Renderer: Debug {
    fn render(&mut self, gfx: &mut Gfx, camera: &Camera, size: PhysicalSize<u32>);
}
