use std::path::{Path, PathBuf};
use wesl::Wesl;

// The engine's shared shader modules, compiled into the `etib` WESL package by
// `build.rs`. Dependent shaders reach them with `import etib::Camera;`.
wesl::wesl_pkg!(etib);

/// Compiles WESL shader modules into `wgpu` shader modules, resolving
/// `import etib::...` against the engine's shader package.
pub struct ShaderBuilder {
    root: PathBuf,
}

impl ShaderBuilder {
    /// `root` is the directory the shader's module path is resolved against, so
    /// `package::foo::bar` maps to `<root>/foo/bar.wgsl`.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// Compile the module at `module_path` (e.g. `package::renderer::static::face_draw`).
    ///
    /// # Errors
    /// Returns an error if the module path is malformed or WESL fails to compile.
    pub fn build_wgsl(
        &self,
        device: &wgpu::Device,
        module_path: &str,
    ) -> Result<wgpu::ShaderModule, Box<dyn std::error::Error>> {
        let wgsl = Wesl::new(&self.root)
            .add_package(&etib::PACKAGE)
            .compile(&module_path.parse()?)?
            .to_string();

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(module_path),
            source: wgpu::ShaderSource::Wgsl(wgsl.into()),
        });
        Ok(module)
    }
}
