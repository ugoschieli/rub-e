use std::fmt::Debug;

use crate::camera::Camera;
use crate::game::EngineContext;

pub mod dynamic_renderer;
pub mod static_renderer;

pub trait Renderer: Debug {
    fn render(&mut self, gfx: &mut EngineContext, camera: &Camera);
}
