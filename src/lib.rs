extern crate assimp;
extern crate chrono;
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

pub mod maths;
pub mod mesh;

use errors::*;
use gfx_hal::window::Extent2D;
use graphics::gfx_hal::backend::BackendState;
use graphics::gfx_hal::image::Loader;
use graphics::gfx_hal::renderer::RendererState;
use graphics::gfx_hal::window::WindowState;

const WINDOW_WIDTH: f64 = 640.0;
const WINDOW_HEIGHT: f64 = 480.0;
const WINDOW_TITLE: &str = "corporation";

#[cfg(any(feature = "gl", feature = "dx12", feature = "vulkan", feature = "metal"))]
pub fn run() -> Result<()> {
  info!("corporation starting...");

  let mut window_state = WindowState::new(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

  let (backend_state, _instance) = BackendState::<gfx_backend::Backend>::new(&mut window_state);

  let framebuffer_extent = Extent2D {
    width: WINDOW_WIDTH as u32,
    height: WINDOW_HEIGHT as u32,
  };

  let mut camera = camera::Camera::<f32>::default();
  camera.set_position(cgmath::Point3::<f32>::new(0.0, 20.0, 15.0));
  camera.look_at(cgmath::Point3::<f32>::new(0.0, 0.0, 0.0), cgmath::Vector3::<f32>::new(0.0, 1.0, 0.0));
  // camera.set_projection_matrix(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32, 90.0, 1.0, 10.0);

  let initializer = graphics::RenderStateInitializer {
    camera,
    textures: vec![(0, Loader::from_file("resources/uv_grid.jpg").expect("Failed to load image"))],
    uniforms: vec![(0, graphics::UniformInitializer {
      data: vec![1.0f32, 0.5f32, 0.5f32, 1.0f32],
    })],
  };

  let mut renderer_state = unsafe { RendererState::new(backend_state, window_state, framebuffer_extent, initializer) };

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
