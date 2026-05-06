//! Experimental raytracing renderer using GPU compute shaders.

/// Camera types and uniforms for the raytracing pipeline
pub mod camera;
/// Cube geometry, materials, and world representation for raytracing
pub mod cube;
/// G-buffer rasterization pass
pub mod rasterizer;
/// Raytracing compute pass and final display pass
pub mod raytracing;
/// Low-level wgpu renderer for the raytracing pipeline
pub mod renderer;
/// Internal wgpu helper utilities for the raytracing pipeline
pub mod utils;

/// Keyboard and mouse button state consumed by the raytracing camera controller.
#[derive(Default)]
pub struct InputState {
    /// Move forward (W / Arrow Up).
    pub forward: bool,
    /// Move backward (S / Arrow Down).
    pub backward: bool,
    /// Strafe left (A / Arrow Left).
    pub left: bool,
    /// Strafe right (D / Arrow Right).
    pub right: bool,
    /// Move up (Space).
    pub up: bool,
    /// Move down (Shift).
    pub down: bool,
    /// Right mouse button held — enables mouse-look.
    pub rmb_pressed: bool,
}

use camera::CameraUniforms;

/// Top-level uniform block shared between the rasterizer and raytracing compute shaders.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    /// Current time in seconds since the Unix epoch.
    pub time: f32,
    /// Accumulated frame counter (reset on camera move).
    pub frame: u32,
    /// 1 if the surface format supports HDR, 0 otherwise.
    pub is_hdr: u32,
    #[allow(missing_docs)]
    pub _padding2: u32,
    /// Camera-specific uniforms (position, ray basis, matrices).
    pub camera_uniforms: CameraUniforms,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_default() {
        let state = InputState::default();
        assert!(!state.forward);
        assert!(!state.backward);
        assert!(!state.left);
        assert!(!state.right);
        assert!(!state.up);
        assert!(!state.down);
        assert!(!state.rmb_pressed);
    }

    #[test]
    fn test_input_state_fields_independent() {
        let mut state = InputState::default();
        state.forward = true;
        state.rmb_pressed = true;
        assert!(state.forward);
        assert!(!state.backward);
        assert!(state.rmb_pressed);
    }

    #[test]
    fn test_uniforms_is_pod() {
        // Verifies Pod/Zeroable are correctly derived — cast to bytes must succeed
        let uniforms = <Uniforms as bytemuck::Zeroable>::zeroed();
        let bytes = bytemuck::bytes_of(&uniforms);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_uniforms_size() {
        // 4 header u32/f32 fields (16 bytes) + CameraUniforms (192 bytes) = 208 bytes
        assert_eq!(std::mem::size_of::<Uniforms>(), 208);
    }
}
