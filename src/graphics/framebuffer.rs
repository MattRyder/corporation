use gfx_hal::format as f;
use gfx_hal::image as i;
use gfx_hal::*;
use graphics::device::DeviceState;
use graphics::image::COLOR_RANGE;
use graphics::renderer::RenderPassState;
use graphics::swapchain::SwapchainState;
use std::cell::RefCell;
use std::rc::Rc;

pub struct FramebufferState<B: Backend> {
    framebuffers: Option<Vec<B::Framebuffer>>,
    framebuffer_fences: Option<Vec<B::Fence>>,
    command_pools: Option<Vec<CommandPool<B, Graphics>>>,
    frame_images: Option<Vec<(B::Image, B::ImageView)>>,
    acquire_semaphores: Option<Vec<B::Semaphore>>,
    present_semaphores: Option<Vec<B::Semaphore>>,
    last_semaphore_index: usize,
    device_state: Rc<RefCell<DeviceState<B, Graphics>>>,
}

impl<B: Backend> FramebufferState<B> {
    pub unsafe fn new(
        device_state: Rc<RefCell<DeviceState<B, Graphics>>>,
        render_pass_state: &RenderPassState<B>,
        swapchain_state: &mut SwapchainState<B>,
    ) -> Self {
        let (frame_images, framebuffers) = match swapchain_state.backbuffer.take().unwrap() {
            Backbuffer::Images(images) => {
                let extent = i::Extent {
                    width: swapchain_state.extent.width,
                    height: swapchain_state.extent.height,
                    depth: 1,
                };

                let pairs = images
                    .into_iter()
                    .map(|image| {
                        let rtv = device_state
                            .as_ref()
                            .borrow()
                            .device
                            .create_image_view(&image, i::ViewKind::D2, swapchain_state.format, f::Swizzle::NO, COLOR_RANGE.clone())
                            .unwrap();

                        (image, rtv)
                    })
                    .collect::<Vec<_>>();

                let fbos = pairs
                    .iter()
                    .map(|&(_, ref rtv)| {
                        device_state
                            .as_ref()
                            .borrow()
                            .device
                            .create_framebuffer(render_pass_state.render_pass.as_ref().unwrap(), Some(rtv), extent)
                            .unwrap()
                    })
                    .collect();

                // (vec![], vec![])
                (pairs, fbos)
            }
            Backbuffer::Framebuffer(fbo) => (Vec::new(), vec![fbo]),
        };

        let iter_count = if frame_images.len() != 0 {
            frame_images.len()
        } else {
            1 // OpenGL can have no frame images
        };

        let mut fences: Vec<B::Fence> = vec![];
        let mut command_pools: Vec<CommandPool<B, Graphics>> = vec![];
        let mut acquire_semaphores: Vec<B::Semaphore> = vec![];
        let mut present_semaphores: Vec<B::Semaphore> = vec![];

        for _ in 0..iter_count {
            fences.push(device_state.as_ref().borrow().device.create_fence(true).unwrap());

            command_pools.push(
                device_state
                    .as_ref()
                    .borrow()
                    .device
                    .create_command_pool_typed(&device_state.as_ref().borrow().queue_group, pool::CommandPoolCreateFlags::empty())
                    .unwrap(),
            );

            acquire_semaphores.push(device_state.as_ref().borrow().device.create_semaphore().unwrap());
            present_semaphores.push(device_state.as_ref().borrow().device.create_semaphore().unwrap());
        }

        FramebufferState {
            command_pools: Some(command_pools),
            acquire_semaphores: Some(acquire_semaphores),
            present_semaphores: Some(present_semaphores),
            framebuffers: Some(framebuffers),
            frame_images: Some(frame_images),
            framebuffer_fences: Some(fences),
            device_state: Rc::clone(&device_state),
            last_semaphore_index: 0,
        }
    }

    /// Returns the next available framebuffer semaphore index
    pub fn get_next_semaphore_index(&mut self) -> usize {
        if self.last_semaphore_index >= self.acquire_semaphores.as_ref().unwrap().len() {
            self.last_semaphore_index = 0;
        }

        let semaphore_index = self.last_semaphore_index;

        self.last_semaphore_index += 1;

        semaphore_index
    }

    pub unsafe fn get_frame_data(&mut self, frame_id: Option<usize>, semaphore_index: Option<usize>) -> (
        Option<(
            &mut B::Fence,
            &mut B::Framebuffer,
            &mut CommandPool<B, Graphics>
        )>,
        Option<(&mut B::Semaphore, &mut B::Semaphore)>
    ) {
        (
            if let Some(frame_id) = frame_id {
                Some((
                    &mut self.framebuffer_fences.as_mut().unwrap()[frame_id],
                    &mut self.framebuffers.as_mut().unwrap()[frame_id],
                    &mut self.command_pools.as_mut().unwrap()[frame_id]
                ))
            } else {
                None
            },
            if let Some(semaphore_index) = semaphore_index {
                Some((
                    &mut self.acquire_semaphores.as_mut().unwrap()[semaphore_index],
                    &mut self.present_semaphores.as_mut().unwrap()[semaphore_index],
                ))
            } else {
                None
            }
        )
    }
}

impl<B: Backend> Drop for FramebufferState<B> {
    fn drop(&mut self) {
        let device = &self.device_state.as_ref().borrow().device;

        unsafe {
            for fence in self.framebuffer_fences.take().unwrap() {
                device.wait_for_fence(&fence, !0).unwrap();
                device.destroy_fence(fence);
            }

            for command_pool in self.command_pools.take().unwrap() {
                device.destroy_command_pool(command_pool.into_raw());
            }

            for acquire_semaphore in self.acquire_semaphores.take().unwrap() {
                device.destroy_semaphore(acquire_semaphore);
            }

            for present_semaphore in self.present_semaphores.take().unwrap() {
                device.destroy_semaphore(present_semaphore);
            }

            for framebuffer in self.framebuffers.take().unwrap() {
                device.destroy_framebuffer(framebuffer);
            }

            for (_, rtv) in self.frame_images.take().unwrap() {
                device.destroy_image_view(rtv);
            }
        }
    }
}
