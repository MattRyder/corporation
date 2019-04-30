use camera::Camera;
use cgmath::InnerSpace;
use cgmath::Zero;
use gfx_hal::image as gfx_image;
use gfx_hal::pso::*;
use gfx_hal::*;
use graphics::gfx_hal::backend::BackendState;
use graphics::gfx_hal::backend::SurfaceTrait;
use graphics::gfx_hal::buffer::BufferState;
use graphics::gfx_hal::descriptor::DescriptorSetLayout;
use graphics::gfx_hal::device::DeviceState;
use graphics::gfx_hal::framebuffer::FramebufferState;
use graphics::gfx_hal::image::ImageState;
use graphics::gfx_hal::input::InputState;
use graphics::gfx_hal::pipeline::PipelineState;
use graphics::gfx_hal::swapchain::SwapchainState;
use graphics::gfx_hal::uniform::Uniform;
use graphics::gfx_hal::window::WindowState;
use graphics::RenderStateInitializer;
use mesh::vertex::Vertex;
use std::cell::RefCell;
use std::rc::Rc;

const CLEAR_COLOR: [f32; 4] = [0.255, 0.412, 0.882, 1.0];

const QUAD: [Vertex; 4] = [
  Vertex {
    a_Position: [-1.0, -1.0, 0.0],
    a_TexCoord: [0.0, 0.0],
  },
  Vertex {
    a_Position: [-1.0, 1.0, 0.0],
    a_TexCoord: [0.0, 1.0],
  },
  Vertex {
    a_Position: [1.0, 1.0, 0.0],
    a_TexCoord: [1.0, 1.0],
  },
  Vertex {
    a_Position: [1.0, -1.0, 0.0],
    a_TexCoord: [1.0, 0.0],
  },
];

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

pub struct RenderPassState<B: Backend> {
  pub render_pass: Option<B::RenderPass>,
  device_state: Rc<RefCell<DeviceState<B, Graphics>>>,
}

impl<B: Backend> RenderPassState<B> {
  pub unsafe fn new(swapchain_state: &SwapchainState<B>, device_state: Rc<RefCell<DeviceState<B, Graphics>>>) -> Self {
    let attachment = pass::Attachment {
      format: Some(swapchain_state.format),
      samples: 1,
      ops: pass::AttachmentOps::new(pass::AttachmentLoadOp::Clear, pass::AttachmentStoreOp::Store),
      stencil_ops: pass::AttachmentOps::DONT_CARE,
      layouts: gfx_image::Layout::Undefined..gfx_image::Layout::Present,
    };

    let subpass = pass::SubpassDesc {
      colors: &[(0, gfx_image::Layout::ColorAttachmentOptimal)],
      depth_stencil: None,
      inputs: &[],
      resolves: &[],
      preserves: &[],
    };

    let dependency = pass::SubpassDependency {
      passes: pass::SubpassRef::External..pass::SubpassRef::Pass(0),
      stages: PipelineStage::COLOR_ATTACHMENT_OUTPUT..PipelineStage::COLOR_ATTACHMENT_OUTPUT,
      accesses: gfx_image::Access::empty()..(gfx_image::Access::COLOR_ATTACHMENT_READ | gfx_image::Access::COLOR_ATTACHMENT_WRITE),
    };

    let render_pass = device_state
      .as_ref()
      .borrow()
      .device
      .create_render_pass(&[attachment], &[subpass], &[dependency])
      .expect("Failed to create render pass");

    RenderPassState {
      render_pass: Some(render_pass),
      device_state: Rc::clone(&device_state),
    }
  }
}

impl<B: Backend> Drop for RenderPassState<B> {
  fn drop(&mut self) {
    let device = &self.device_state.as_ref().borrow().device;
    unsafe {
      device.destroy_render_pass(self.render_pass.take().unwrap());
    }
  }
}

pub struct RendererState<B: Backend> {
  backend_state: BackendState<B>,
  pub device_state: Rc<RefCell<DeviceState<B, Graphics>>>,
  framebuffer_state: FramebufferState<B>,
  descriptor_pool: Option<B::DescriptorPool>,
  index_buffer: BufferState<B, Graphics>,
  pipeline_state: PipelineState<B>,
  render_pass_state: RenderPassState<B>,
  swapchain_state: Option<SwapchainState<B>>,
  viewport: pso::Viewport,
  vertex_buffer: BufferState<B, Graphics>,
  window_state: WindowState,

  // New fields
  uniforms: Vec<Uniform<B>>,
  image_states: Vec<ImageState<B>>,
  camera: Camera<f32>,
  input_state: InputState,
}

impl<B: Backend> RendererState<B> {
  pub unsafe fn new<T: Copy>(
    mut backend_state: BackendState<B>,
    window_state: WindowState,
    frame_extent: window::Extent2D,
    initializer: RenderStateInitializer<T>,
  ) -> Self {
    let device_state = Rc::new(RefCell::new(DeviceState::new(
      backend_state.adapter_state.adapter.take().unwrap(),
      &backend_state.surface,
    )));

    let vertex_buffer = BufferState::new::<Vertex>(
      Rc::clone(&device_state),
      &QUAD,
      buffer::Usage::VERTEX,
      &backend_state.adapter_state.mem_types,
    );

    let index_buffer = BufferState::new::<u16>(
      Rc::clone(&device_state),
      &QUAD_INDICES,
      buffer::Usage::INDEX,
      &backend_state.adapter_state.mem_types,
    );

    let descriptor_pool_max_sets = 1;

    let mut descriptor_pool = device_state
      .borrow()
      .device
      .create_descriptor_pool(
        descriptor_pool_max_sets,
        &[
          pso::DescriptorRangeDesc {
            count: 1,
            ty: pso::DescriptorType::SampledImage,
          },
          pso::DescriptorRangeDesc {
            count: 1,
            ty: pso::DescriptorType::Sampler,
          },
          pso::DescriptorRangeDesc {
            count: 2,
            ty: pso::DescriptorType::UniformBuffer,
          },
        ],
      )
      .ok();

    let mut staging_pool = device_state
      .as_ref()
      .borrow()
      .device
      .create_command_pool_typed(&device_state.as_ref().borrow().queue_group, pool::CommandPoolCreateFlags::empty())
      .unwrap();

    let mut image_states = vec![];

    for texture in initializer.textures {
      let image_desc_set_layout = DescriptorSetLayout::new(
        Rc::clone(&device_state),
        vec![
          pso::DescriptorSetLayoutBinding {
            binding: 0,
            ty: pso::DescriptorType::SampledImage,
            stage_flags: pso::ShaderStageFlags::FRAGMENT,
            count: 1,
            immutable_samplers: false,
          },
          pso::DescriptorSetLayoutBinding {
            binding: 1,
            ty: pso::DescriptorType::Sampler,
            stage_flags: pso::ShaderStageFlags::FRAGMENT,
            count: 1,
            immutable_samplers: false,
          },
        ],
      );

      let image_descriptor_set = image_desc_set_layout.create_set(descriptor_pool.as_mut().unwrap());

      let image_state = ImageState::new(
        image_descriptor_set,
        &mut device_state.borrow_mut(),
        &backend_state.adapter_state,
        texture.1,
        buffer::Usage::TRANSFER_SRC,
        &mut staging_pool,
      );

      image_state.wait_for_transfer();

      image_states.push(image_state);
    }

    device_state.as_ref().borrow().device.destroy_command_pool(staging_pool.into_raw());

    let mut uniforms = vec![];

    let camera_desc_set_layout = DescriptorSetLayout::new(
      Rc::clone(&device_state),
      vec![pso::DescriptorSetLayoutBinding {
        binding: 0,
        ty: pso::DescriptorType::UniformBuffer,
        stage_flags: pso::ShaderStageFlags::VERTEX,
        count: 1,
        immutable_samplers: false,
      }],
    );

    let cam_desc_set = camera_desc_set_layout.create_set(descriptor_pool.as_mut().unwrap());

    let camera_uniform = Uniform::new(
      Rc::clone(&device_state),
      &backend_state.adapter_state.mem_types,
      &initializer.camera.get_mvp_matrix_array(),
      cam_desc_set,
      0,
    );

    uniforms.push(camera_uniform);

    for (binding_idx, uniform_data) in initializer.uniforms {
      let unif_desc_set_layout = DescriptorSetLayout::new(
        Rc::clone(&device_state),
        vec![pso::DescriptorSetLayoutBinding {
          binding: binding_idx,
          ty: pso::DescriptorType::UniformBuffer,
          stage_flags: pso::ShaderStageFlags::FRAGMENT,
          count: 1,
          immutable_samplers: false,
        }],
      );

      let desc_set = unif_desc_set_layout.create_set(descriptor_pool.as_mut().unwrap());

      let uniform = Uniform::new(
        Rc::clone(&device_state),
        &backend_state.adapter_state.mem_types,
        &uniform_data.data,
        desc_set,
        0,
      );

      uniforms.push(uniform);
    }

    let swapchain_state = SwapchainState::new(&mut backend_state, Rc::clone(&device_state), frame_extent);

    let mut swapchain_state = Some(swapchain_state);

    let render_pass_state = RenderPassState::new(swapchain_state.as_ref().unwrap(), Rc::clone(&device_state));

    let framebuffer_state = FramebufferState::new(Rc::clone(&device_state), &render_pass_state, swapchain_state.as_mut().unwrap());

    let pipeline_state = {
      let uniform_desc_sets = uniforms.iter().map(|u| u.get_layout()).collect::<Vec<&B::DescriptorSetLayout>>();

      let mut image_desc_sets = image_states
        .iter()
        .map(|i| i.get_layout())
        .collect::<Vec<&B::DescriptorSetLayout>>();

      image_desc_sets.extend(uniform_desc_sets);

      PipelineState::new(
        image_desc_sets,
        render_pass_state.render_pass.as_ref().unwrap(),
        Rc::clone(&device_state),
      )
    };

    let viewport = Self::create_viewport(&swapchain_state.as_ref().unwrap());

    let input_state = InputState::new();

    RendererState {
      backend_state,
      device_state,
      framebuffer_state,
      descriptor_pool,
      index_buffer,
      input_state,
      pipeline_state,
      render_pass_state,
      swapchain_state,
      window_state,
      vertex_buffer,
      viewport,

      uniforms,
      image_states,
      camera: initializer.camera,
    }
  }

  pub unsafe fn render(&mut self)
  where
    B::Surface: SurfaceTrait,
  {
    let mut is_running = true;
    let mut will_recreate_swapchain = false;

    let mut current_frame_time_ms = chrono::Utc::now().timestamp_millis();

    let mut resize_dimensions = window::Extent2D { width: 640, height: 480 };

    let mut keyboard_input: Option<winit::KeyboardInput> = None;

    while is_running {
      let new_frame_time_ms = chrono::Utc::now().timestamp_millis();
      let delta_time_ms = new_frame_time_ms - current_frame_time_ms;
      current_frame_time_ms = new_frame_time_ms;

      {
        #[cfg(feature = "gl")]
        let backend = &self.backend_state;

        // Handles the window event loop:
        self.window_state.event_loop.poll_events(|event| {
          if let winit::Event::WindowEvent { event, .. } = event {
            match event {
              // Handle the window being closed:
              winit::WindowEvent::CloseRequested => is_running = false,

              // Handle the window being resized:
              winit::WindowEvent::Resized(dimensions) => {
                info!("Window Resized: {:?}", dimensions);

                #[cfg(feature = "gl")]
                backend
                  .surface
                  .get_window_t()
                  .resize(dimensions.to_physical(backend.surface.get_window_t().get_hidpi_factor()));

                will_recreate_swapchain = true;

                resize_dimensions.width = dimensions.width as u32;
                resize_dimensions.height = dimensions.height as u32;
              }

              winit::WindowEvent::KeyboardInput { input, .. } => {
                keyboard_input = Some(input);
              }
              _ => {}
            }
          }
        });
      }

      if will_recreate_swapchain {
        //self.recreate_swapchain(resize_dimensions);
        will_recreate_swapchain = false;
      }

      // hack in input state support here, fix later:
      if let Some(kb_input) = keyboard_input {
        self.input_state.on_key_input(kb_input.virtual_keycode.unwrap(), kb_input.state);
      }

      // Handle camera updates via input here:
      let movement_speed = 1.0 / delta_time_ms as f32;
      let mut camera_movement = cgmath::Vector3::<f32>::zero();
      if self.input_state.is_key_down(&winit::VirtualKeyCode::Up) {
        camera_movement.z = movement_speed;
      } else if self.input_state.is_key_down(&winit::VirtualKeyCode::Down) {
        camera_movement.z = -movement_speed;
      }

      if self.input_state.is_key_down(&winit::VirtualKeyCode::Left) {
        camera_movement.x = -movement_speed;
      } else if self.input_state.is_key_down(&winit::VirtualKeyCode::Right) {
        camera_movement.x = movement_speed;
      }

      if camera_movement.magnitude() > 0.0 {
        let cam_pos = self.camera.get_position();
        self.camera.set_position(cam_pos + camera_movement);
        self
          .uniforms
          .first_mut()
          .unwrap()
          .buffer_state
          .as_mut()
          .unwrap()
          .update_buffer(0, &self.camera.get_mvp_matrix_array());
      }

      let semaphore_index = self.framebuffer_state.get_next_semaphore_index();

      let frame: SwapImageIndex = {
        let (acquire_semaphore, _) = self.framebuffer_state.get_frame_data(None, Some(semaphore_index)).1.unwrap();

        let swapchain = self.swapchain_state.as_mut().unwrap().swapchain.as_mut().unwrap();

        match swapchain.acquire_image(!0, gfx_hal::FrameSync::Semaphore(acquire_semaphore)) {
          Ok(img) => img,
          Err(_) => {
            will_recreate_swapchain = true;
            continue;
          }
        }
      };

      let (frame_data, semaphore_data) = self.framebuffer_state.get_frame_data(Some(frame as usize), Some(semaphore_index));

      let (framebuffer_fence, framebuffer, command_pool) = frame_data.unwrap();
      let (acquire_semaphore, present_semaphore) = semaphore_data.unwrap();

      self
        .device_state
        .as_ref()
        .borrow()
        .device
        .wait_for_fence(&framebuffer_fence, !0)
        .unwrap();

      self.device_state.as_ref().borrow().device.reset_fence(&framebuffer_fence).unwrap();

      command_pool.reset();

      let mut cmd_buffer = command_pool.acquire_command_buffer::<command::OneShot>();
      cmd_buffer.begin();

      let uniform_desc_sets = self
        .uniforms
        .iter()
        .map(|u| u.descriptor_set.as_ref().unwrap().set.as_ref().unwrap())
        .collect::<Vec<&B::DescriptorSet>>();

      let mut image_desc_sets = self
        .image_states
        .iter()
        .map(|i| i.descriptor_set.set.as_ref().unwrap())
        .collect::<Vec<&B::DescriptorSet>>();

      image_desc_sets.extend(uniform_desc_sets);

      // Record a command buffer to get some rendering going:
      cmd_buffer.set_viewports(0, &[self.viewport.clone()]);
      cmd_buffer.set_scissors(0, &[self.viewport.rect]);
      cmd_buffer.bind_graphics_pipeline(self.pipeline_state.pipeline.as_ref().unwrap());
      cmd_buffer.bind_vertex_buffers(0, Some((self.vertex_buffer.buffer.as_ref().unwrap(), 0)));
      cmd_buffer.bind_index_buffer(self.index_buffer.get_buffer_view());

      cmd_buffer.bind_graphics_descriptor_sets(self.pipeline_state.pipeline_layout.as_ref().unwrap(), 0, image_desc_sets, &[]);

      {
        let mut encoder = cmd_buffer.begin_render_pass_inline(
          self.render_pass_state.render_pass.as_ref().unwrap(),
          &framebuffer,
          self.viewport.rect,
          &[command::ClearValue::Color(command::ClearColor::Float(CLEAR_COLOR.clone()))],
        );

        encoder.draw_indexed(0..6, 0, 0..1);
      }

      cmd_buffer.finish();

      // Tell GPU we're doing a command buffer:
      let submission = Submission {
        command_buffers: std::iter::once(&cmd_buffer),
        wait_semaphores: std::iter::once((&*acquire_semaphore, PipelineStage::BOTTOM_OF_PIPE)),
        signal_semaphores: std::iter::once(&*present_semaphore),
      };

      self.device_state.as_ref().borrow_mut().queue_group.queues[0].submit(submission, Some(framebuffer_fence));

      if let Err(_) = self.swapchain_state.as_ref().unwrap().swapchain.as_ref().unwrap().present(
        &mut self.device_state.as_ref().borrow_mut().queue_group.queues[0],
        frame,
        Some(&*present_semaphore),
      ) {
        // Failed to present image, swapchain should be rebuilt:
        will_recreate_swapchain = true;
        continue;
      }
    }
  }

  fn handle_input() {}

  unsafe fn recreate_swapchain(&mut self, frame_extent: window::Extent2D) {
    let device = &self.device_state.as_ref().borrow().device;

    device.wait_idle().unwrap();

    self.swapchain_state.take().unwrap();

    let new_swapchain_state = SwapchainState::new(&mut self.backend_state, Rc::clone(&self.device_state), frame_extent);

    self.swapchain_state = Some(new_swapchain_state);

    self.render_pass_state = RenderPassState::new(&self.swapchain_state.as_ref().unwrap(), Rc::clone(&self.device_state));

    self.framebuffer_state = FramebufferState::new(
      Rc::clone(&self.device_state),
      &self.render_pass_state,
      self.swapchain_state.as_mut().unwrap(),
    );

    let uniform_desc_sets = self
      .uniforms
      .iter()
      .map(|u| u.get_layout())
      .collect::<Vec<&B::DescriptorSetLayout>>();

    let mut image_desc_sets = self
      .image_states
      .iter()
      .map(|i| i.get_layout())
      .collect::<Vec<&B::DescriptorSetLayout>>();

    image_desc_sets.extend(uniform_desc_sets);

    self.pipeline_state = PipelineState::new(
      image_desc_sets,
      self.render_pass_state.render_pass.as_ref().unwrap(),
      Rc::clone(&self.device_state),
    );

    self.viewport = Self::create_viewport(self.swapchain_state.as_ref().unwrap());
  }

  fn create_viewport(swapchain_state: &SwapchainState<B>) -> pso::Viewport {
    Viewport {
      rect: pso::Rect {
        x: 0,
        y: 0,
        w: swapchain_state.extent.width as _,
        h: swapchain_state.extent.width as _,
      },
      depth: 0.0..1.0,
    }
  }
}

impl<B: Backend> Drop for RendererState<B> {
  fn drop(&mut self) {
    self.device_state.as_ref().borrow().device.wait_idle().unwrap();

    unsafe {
      self
        .device_state
        .as_ref()
        .borrow()
        .device
        .destroy_descriptor_pool(self.descriptor_pool.take().unwrap());

      self.swapchain_state.take();
    }
  }
}
