use gfx_hal::format::AsFormat;
use gfx_hal::format::Aspects;
use gfx_hal::format::D32Float;
use gfx_hal::image as i;
use gfx_hal::image::Extent;
use gfx_hal::*;
use graphics::gfx_hal::adapter::AdapterState;
use graphics::gfx_hal::device::DeviceState;
use graphics::gfx_hal::image::ImageResourceState;
use std::cell::RefCell;
use std::rc::Rc;

pub type DepthFormat = D32Float;

pub const DEPTH_RANGE: i::SubresourceRange = i::SubresourceRange {
    aspects: Aspects::DEPTH,
    levels: 0..1,
    layers: 0..1,
};

pub struct DepthImageState<B: Backend> {
    image_resource_state: Option<ImageResourceState<B>>,
}

impl<B: Backend> DepthImageState<B> {
    pub unsafe fn new(
        non_device_state: &DeviceState<B, Graphics>,
        device_state: Rc<RefCell<DeviceState<B, Graphics>>>,
        adapter_state: &AdapterState<B>,
        extent: Extent,
    ) -> Self {
        let depth_kind = i::Kind::D2(extent.width, extent.height, 1, 1);

        let image_resource_state = ImageResourceState::new(
            non_device_state,
            adapter_state,
            DEPTH_RANGE,
            Rc::clone(&device_state),
            &depth_kind,
            i::Usage::DEPTH_STENCIL_ATTACHMENT,
            DepthFormat::SELF,
        );

        DepthImageState {
            image_resource_state: Some(image_resource_state),
        }
    }

    pub fn get_image_view(&self) -> Option<&B::ImageView> {
        self.image_resource_state.as_ref().unwrap().image_view.as_ref()
    }
}
