use naga::{self, valid};

/// Shader wrapper struct of a wgpu ShaderModule
pub struct Shader {
    /// The wgpu ShaderModule
    pub module: wgpu::ShaderModule,
}

impl Shader {
    /// Create a new wgsl shader from the source string.
    /// The function panics if the shader is invalid.
    pub fn new(source: &str, device: &wgpu::Device, label: Option<&str>) -> Shader {
        let module = naga::front::wgsl::parse_str(source).unwrap_or_else(|err| {
            log::error!("Failed to parse shader source: {}", err);
            panic!()
        });

        let mut validator =
            valid::Validator::new(valid::ValidationFlags::all(), valid::Capabilities::all());

        validator.validate(&module).unwrap_or_else(|err| {
            log::error!("Failed to validate shader: {}", err);
            panic!()
        });

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        Shader { module }
    }
}
