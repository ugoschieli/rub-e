use etib::camera::Camera;
use etib::constants::{CUBE_NUMBER, CUBE_RANGE};
use etib::cube::Cube;
use etib::game::{EngineContext, Game};
use etib::renderer::Renderer;
use etib::renderer::dynamic_renderer::DynamicRenderer;
use etib::renderer::static_renderer::StaticRenderer;
use etib::updater::line_updater::LineUpdater;
use etib::updater::orbit_updater::OrbitUpdater;
use etib::updater::{TransformBuffers, Updater};
use winit::event::DeviceEvent;

struct Hello {
    cubes: Vec<Cube>,
    camera: Camera,
    transforms: TransformBuffers,
    updaters: Vec<Box<dyn Updater>>,
    renderers: Vec<Box<dyn Renderer>>,
}

impl Game for Hello {
    type InitParams = ();

    fn init(ctx: &mut EngineContext, _params: Self::InitParams) -> Self {
        ctx.set_cursor_visible(false);

        let cubes = (0..CUBE_NUMBER)
            .map(|_| Cube::random_cube(CUBE_RANGE))
            .collect::<Vec<Cube>>();

        let camera = Camera::new(&ctx.gfx);

        // Shared transform buffers the updaters write and the renderer reads.
        let transforms = TransformBuffers::new(&ctx.gfx, &cubes);

        // Split the cubes in half: the first half orbits, the second oscillates
        // along a line. Each updater writes its own contiguous transform range.
        let half = cubes.len() / 2;

        let renderers: Vec<Box<dyn Renderer>> = vec![
            Box::new(StaticRenderer::init(&ctx.gfx, &camera)),
            Box::new(DynamicRenderer::init(
                &ctx.gfx,
                &camera,
                &cubes,
                transforms.buffers(),
            )),
        ];

        let updaters: Vec<Box<dyn Updater>> = vec![
            Box::new(OrbitUpdater::init(
                &ctx.gfx,
                &cubes[..half],
                transforms.buffers(),
                0,
            )),
            Box::new(LineUpdater::init(
                &ctx.gfx,
                &cubes[half..],
                transforms.buffers(),
                half as u32,
            )),
        ];

        Self {
            cubes,
            camera,
            transforms,
            renderers,
            updaters,
        }
    }

    fn update(&mut self, ctx: &mut EngineContext) {
        self.camera.handle_keyboard(&ctx.input, &ctx.time);

        if ctx.gfx.surface_texture.is_some() {
            self.camera.upload(&ctx.gfx, ctx.window_size());

            // Updaters write this frame's transforms before any renderer
            // reads them (the pass boundary is the memory barrier).
            for updater in &mut self.updaters {
                updater.update(&mut ctx.gfx);
            }

            for renderer in &mut self.renderers {
                renderer.render(ctx, &self.camera);
            }

            ctx.gfx.submit();
        }
    }

    fn device_input(&mut self, _ctx: &mut EngineContext, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.camera.handle_mouse(*delta);
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    etib::game::run::<Hello>(None, Some(()))?;

    Ok(())
}
