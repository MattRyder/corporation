use gfx_hal::format as gfx_format;
use gfx_hal::format::{AsFormat, ChannelType};
use gfx_hal::image as gfx_image;
use gfx_hal::*;
use graphics::backend::{BackendState, ColorFormat};
use graphics::device::DeviceState;
use std::cell::RefCell;
use std::rc::Rc;

const DEFAULT_EXTENT : window::Extent2D = window::Extent2D {
    width: 640,
    height: 480
};

pub struct SwapchainState<B: Backend> {
    pub swapchain: Option<B::Swapchain>,
    pub backbuffer: Option<Backbuffer<B>>,
    device_state: Rc<RefCell<DeviceState<B, Graphics>>>,
    pub extent: gfx_image::Extent,
    pub format: gfx_format::Format,
}

impl<B: Backend> SwapchainState<B> {
    pub unsafe fn new(
        backend_state: &mut BackendState<B>,
        device_state: Rc<RefCell<DeviceState<B, Graphics>>>,
        frame_extent: window::Extent2D,
    ) -> Self {
        let (caps, formats, _present_modes, _comp_alpha) = backend_state.surface.compatibility(&device_state.as_ref().borrow().phys_device);

        info!("Formats: {:?}", &formats);

        let format = formats.map_or(ColorFormat::SELF, |formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .map(|format| *format)
                .unwrap_or(formats[0])
        });

        let swap_config = SwapchainConfig::from_caps(&caps, format, frame_extent);

        // Get the framebuffer extent of the swapchain generated
        let extent = swap_config.extent.to_extent();

        info!("Swapchain Config: {:?}", &swap_config);

        let (swapchain, backbuffer) = device_state
            .as_ref()
            .borrow()
            .device
            .create_swapchain(&mut backend_state.surface, swap_config, None)
            .expect("Failed to create swapchain!");

        SwapchainState {
            swapchain: Some(swapchain),
            backbuffer: Some(backbuffer),
            device_state,
            extent,
            format,
        }
    }
}
