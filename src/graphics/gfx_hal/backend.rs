use gfx_hal::format::{AsFormat, Rgba8Srgb};
use gfx_hal::*;
use graphics::gfx_hal::adapter::AdapterState;
use graphics::gfx_hal::window::WindowState;

pub type ColorFormat = Rgba8Srgb;

pub trait SurfaceTrait {
  #[cfg(feature = "gl")]
  fn get_window_t(&self) -> &gfx_backend::glutin::GlWindow;
}

impl SurfaceTrait for <gfx_backend::Backend as gfx_hal::Backend>::Surface {
  #[cfg(feature = "gl")]
  fn get_window_t(&self) -> &gfx_backend::glutin::GlWindow {
    self.get_window()
  }
}

pub struct BackendState<B: Backend> {
  pub surface: B::Surface,
  pub adapter_state: AdapterState<B>,

  #[cfg(any(feature = "vulkan", feature = "dx12", feature = "metal"))]
  pub window: winit::Window,
}

impl<B: Backend> BackendState<B> {

  #[cfg(any(feature = "vulkan", feature = "dx12", feature = "metal"))]
  pub fn new(window_state: &mut WindowState) -> (BackendState<gfx_backend::Backend>, gfx_backend::Instance) {
    let window = window_state.window_builder.take().unwrap().build(&window_state.event_loop).unwrap();

    let instance = gfx_backend::Instance::create("libcorporation", 1);
    let surface = instance.create_surface(&window);
    let mut adapters = instance.enumerate_adapters();

    let backend_state = BackendState {
      adapter_state: AdapterState::new(&mut adapters),
      surface,
      window,
    };

    (backend_state, instance)
  }

  #[cfg(feature = "gl")]
  pub fn new(window_state: &mut WindowState) -> (BackendState<gfx_backend::Backend>, ()) {
    let window = {
      let builder = gfx_backend::config_context(gfx_backend::glutin::ContextBuilder::new(), ColorFormat::SELF, None).with_vsync(true);

      gfx_backend::glutin::GlWindow::new(
        window_state.window_builder.take().unwrap(),
        builder,
        window_state.borrow_event_loop(),
      ).unwrap()
    };

    let surface = gfx_backend::Surface::from_window(window);
    let mut adapters = surface.enumerate_adapters();

    let backend_state = BackendState {
      adapter_state: AdapterState::new(&mut adapters),
      surface
    };

    (backend_state, ())
  }

  #[cfg(feature = "empty")]
  pub fn new(window_state: &mut WindowState) {

  }
}
