extern crate assimp;
extern crate cgmath;
#[macro_use]
extern crate error_chain;
extern crate image;
extern crate shaderc;
#[macro_use]
extern crate log;
extern crate winit;

extern crate gfx_hal as gfx_hal;
#[cfg(feature = "empty")]
extern crate gfx_backend_empty as gfx_backend;
#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as gfx_backend;
#[cfg(feature = "metal")]
extern crate gfx_backend_metal as gfx_backend;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as gfx_backend;
#[cfg(feature = "gl")]
extern crate gfx_backend_gl as gfx_backend;

pub mod camera;
pub mod errors;
pub mod graphics;
pub mod mesh;
pub mod scene;

use graphics::context::Builder;
use errors::*;

const WINDOW_WIDTH: f64 = 640.0;
const WINDOW_HEIGHT: f64 = 480.0;
const WINDOW_TITLE: &str = "corporation";

pub fn run() -> Result<()> {
    info!("corporation starting...");

    Builder::build(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

    Ok(())
}