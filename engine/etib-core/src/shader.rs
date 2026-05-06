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

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_SHADER: &str = concat!(
        "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); } ",
        "@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }",
    );

    fn make_device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));
        if let Ok(adapter) = adapter {
            Some(
                pollster::block_on(
                    adapter.request_device(&wgpu::DeviceDescriptor::default()),
                )
                .unwrap(),
            )
        } else {
            None
        }
    }

    #[test]
    fn test_shader_new_valid_wgsl() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let _shader = Shader::new(MINIMAL_SHADER, &device, Some("test shader"));
    }

    #[test]
    fn test_shader_new_no_label() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let _shader = Shader::new(MINIMAL_SHADER, &device, None);
    }

    #[test]
    fn test_naga_accepts_valid_wgsl() {
        let result = naga::front::wgsl::parse_str(MINIMAL_SHADER);
        assert!(result.is_ok());
    }

    #[test]
    fn test_naga_rejects_invalid_wgsl() {
        let result = naga::front::wgsl::parse_str("this is @@ not valid wgsl !!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_naga_validates_shader() {
        let module = naga::front::wgsl::parse_str(MINIMAL_SHADER).unwrap();
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        assert!(validator.validate(&module).is_ok());
    }

    #[test]
    fn test_naga_rejects_invalid_types() {
        // Valid syntax but semantically invalid: return type mismatch
        let bad_shader = "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec2<f32>(0.0); }";
        let parse_result = naga::front::wgsl::parse_str(bad_shader);
        // Either fails to parse or fails validation
        if let Ok(module) = parse_result {
            let mut validator = naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::all(),
            );
            assert!(validator.validate(&module).is_err());
        }
    }

    /// Shader::new panics (lines 14-15) when the WGSL source cannot be parsed.
    #[test]
    fn test_shader_new_invalid_parse_panics() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Shader::new("@@@@ completely invalid @@@", &device, None);
        }));
        assert!(result.is_err(), "Shader::new must panic on unparseable WGSL");
    }

    /// Shader::new panics (lines 22-23) when naga validation fails.
    /// This shader parses but has a semantic type error that naga rejects.
    #[test]
    fn test_shader_new_invalid_validation_panics() {
        let Some((device, _)) = make_device() else {
            return;
        };
        // This shader is syntactically valid WGSL but semantically wrong:
        // the vertex entry-point must return a struct/builtin position, not void.
        let bad_shader = "fn helper() { } @vertex fn vs_main() { }";
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Shader::new(bad_shader, &device, None);
        }));
        // If the device exists, this MUST panic at validation or at wgpu shader creation
        assert!(result.is_err(), "Shader::new must panic on invalid shader");
    }
}
