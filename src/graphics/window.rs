use winit::dpi::LogicalSize;
use winit::{EventsLoop, WindowBuilder};

/// Handles the state of the window and any changes
/// that arise from modifications to the window or
/// events raised by the system
pub struct WindowState {
  pub event_loop: winit::EventsLoop,
  pub window_builder: Option<winit::WindowBuilder>,
}

impl WindowState {

  pub fn new(title: &str, width: f64, height: f64) -> Self {
    let event_loop = EventsLoop::new();

    let window_builder = WindowBuilder::new()
      .with_dimensions(LogicalSize::new(width, height))
      .with_title(title);

    WindowState {
      event_loop,
      window_builder: Some(window_builder),
    }
  }

  pub fn borrow_event_loop(&self) -> &winit::EventsLoop {
    &self.event_loop
  }

  pub fn borrow_event_loop_mut(&mut self) -> &mut winit::EventsLoop {
    &mut self.event_loop
  }
}
