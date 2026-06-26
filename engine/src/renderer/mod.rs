use std::fmt::Debug;

use crate::core::surface::Frame;
use crate::game::EngineContext;

pub mod dynamic_renderer;
pub mod static_renderer;

/// Root directory WESL module paths are resolved against, e.g.
/// `package::renderer::static::face_draw` -> `<SHADER_DIR>/renderer/static/face_draw.wgsl`.
pub const SHADER_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/renderer");

pub trait Renderer: Debug {
    fn render(&mut self, gfx: &mut EngineContext, frame: &mut Frame);
}
