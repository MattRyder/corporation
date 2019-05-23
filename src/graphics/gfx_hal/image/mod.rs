use gfx_hal::format as f;
use gfx_hal::format::AsFormat;
use gfx_hal::image as i;
use gfx_hal::memory as m;
use gfx_hal::*;
use graphics::gfx_hal::adapter::AdapterState;
use graphics::gfx_hal::backend::ColorFormat;
use graphics::gfx_hal::buffer::BufferState;
use graphics::gfx_hal::device::DeviceState;
use image;
use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

pub mod depth;
pub mod texture;

/// The number of bytes per value of an RGBA image
pub const RGBA_IMAGE_STRIDE: usize = 4;

pub struct Image {
    pub image: image::RgbaImage,
    pub kind: i::Kind,
}

impl Image {
    pub fn get_image(&self) -> &image::RgbaImage {
        &self.image
    }

    pub fn get_kind(&self) -> &i::Kind {
        &self.kind
    }

    pub fn get_dimensions(&self) -> (u32, u32) {
        self.image.dimensions()
    }

    pub fn row_pitch(&self, row_alignment_mask: u32) -> u32 {
        let (width, _) = self.get_dimensions();
        (width * RGBA_IMAGE_STRIDE as u32 + row_alignment_mask) & !row_alignment_mask
    }

    /// Returns a range which denotes the start and finish of bytes
    /// that belong to the row provided by row_index
    pub fn row_range(&self, row_index: usize) -> Range<usize> {
        let (width, _) = self.get_dimensions();
        let width = width as usize;

        let row_index_start = row_index * width * RGBA_IMAGE_STRIDE;
        let row_index_end = (row_index + 1) * width * RGBA_IMAGE_STRIDE;

        row_index_start..row_index_end
    }

    /// Returns the complete size of the image
    pub fn get_upload_size(&self, row_alignment_mask: u32) -> u64 {
        let (_, height) = self.get_dimensions();
        let row_pitch = self.row_pitch(row_alignment_mask);
        (height * row_pitch) as u64
    }
}

pub struct Loader;

impl Loader {
    /// Loads a texture from file
    pub fn from_file(image_file_path: &str) -> Option<Image> {
        match image::open(image_file_path) {
            Ok(image) => {
                let image = image.to_rgba();
                let (width, height) = image.dimensions();
                let kind = i::Kind::D2(width, height, 1, 1);

                Some(Image { image, kind })
            }
            Err(_) => None,
        }
    }
}

pub struct ImageResourceState<B: Backend> {
    device_state: Rc<RefCell<DeviceState<B, Graphics>>>,
    image: Option<B::Image>,
    pub image_view: Option<B::ImageView>,
    memory: Option<B::Memory>,
}

impl<B: Backend> ImageResourceState<B> {
    pub unsafe fn new(
        non_device_state: &DeviceState<B, Graphics>,
        adapter_state: &AdapterState<B>,
        subresource_range: i::SubresourceRange,
        device_state: Rc<RefCell<DeviceState<B, Graphics>>>,
        kind: &i::Kind,
        usage: i::Usage,
        color_format: f::Format,
    ) -> Self {
        const MIP_LEVELS: u8 = 1;

        let mut image = non_device_state
            .device
            .create_image(
                kind.clone(),
                MIP_LEVELS,
                color_format,
                i::Tiling::Optimal,
                usage,
                i::ViewCapabilities::empty(),
            )
            .unwrap();

        let device = &non_device_state.device;

        let requirements = device.get_image_requirements(&image);

        let device_memory_type =
            BufferState::<B, Graphics>::find_buffer_memory(&adapter_state.mem_types, &requirements, m::Properties::DEVICE_LOCAL);

        let device_image_memory = device.allocate_memory(device_memory_type, requirements.size).unwrap();

        device.bind_image_memory(&device_image_memory, 0, &mut image).unwrap();

        let image_view = device
            .create_image_view(
                &image,
                i::ViewKind::D2,
                ColorFormat::SELF,
                f::Swizzle::NO,
                subresource_range.clone(),
            )
            .unwrap();

        Self {
            image: Some(image),
            image_view: Some(image_view),
            memory: Some(device_image_memory),
            device_state: Rc::clone(&device_state),
        }
    }
}

impl<B: Backend> Drop for ImageResourceState<B> {
    fn drop(&mut self) {
        let device = &self.device_state.as_ref().borrow().device;

        unsafe {
            device.destroy_image_view(self.image_view.take().unwrap());
            device.destroy_image(self.image.take().unwrap());

            device.free_memory(self.memory.take().unwrap());
        }
    }
}
