extern crate assimp;
extern crate cgmath;
#[macro_use]
extern crate error_chain;
extern crate image;
extern crate shaderc;
#[macro_use]
extern crate log;
extern crate winit;

#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as gfx_backend;
#[cfg(feature = "empty")]
extern crate gfx_backend_empty as gfx_backend;
#[cfg(feature = "gl")]
extern crate gfx_backend_gl as gfx_backend;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as gfx_backend;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as gfx_backend;
extern crate gfx_hal as gfx_hal;

pub mod camera;
pub mod errors;
pub mod graphics;
pub mod mesh;
pub mod scene;

use errors::*;
use graphics::backend::BackendState;
use graphics::renderer::RendererState;
use graphics::window::WindowState;

const WINDOW_WIDTH: f64 = 640.0;
const WINDOW_HEIGHT: f64 = 480.0;
const WINDOW_TITLE: &str = "corporation";

#[cfg(any(feature = "gl", feature = "dx12", feature = "vulkan", feature = "metal"))]
pub fn run() -> Result<()> {
  info!("corporation starting...");

  let mut window_state = WindowState::new(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

  let (backend_state, _instance) = BackendState::<gfx_backend::Backend>::new(&mut window_state);

  let mut renderer_state = unsafe { RendererState::new(backend_state, window_state, WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32) };

  unsafe {
    renderer_state.render();
  }

  Ok(())
}

#[cfg(feature = "empty")]
pub fn run() -> Result<()> {
  error!("corporation requires a non-empty gfx_hal backend");

  Ok(())
}
