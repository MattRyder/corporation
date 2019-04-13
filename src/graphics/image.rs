use gfx_hal::format as f;
use gfx_hal::format::AsFormat;
use gfx_hal::image as i;
use gfx_hal::memory as m;
use gfx_hal::pso::PipelineStage;
use gfx_hal::*;
use graphics::adapter::AdapterState;
use graphics::backend::ColorFormat;
use graphics::buffer::BufferState;
use graphics::descriptor::{DescriptorSet, DescriptorSetLayout, DescriptorSetWrite};
use graphics::device::DeviceState;
use image;
use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

/// The number of bytes per value of an RGBA image
pub const RGBA_IMAGE_STRIDE: usize = 4;

pub const COLOR_RANGE: i::SubresourceRange = i::SubresourceRange {
    aspects: f::Aspects::COLOR,
    levels: 0..1,
    layers: 0..1,
};

pub struct ImageState<B: Backend> {
    pub descriptor_set: DescriptorSet<B, Graphics>,
    sampler: Option<B::Sampler>,
    buffer: Option<B::Buffer>,
    image_view: Option<B::ImageView>,
    image: Option<B::Image>,
    memory: Option<B::Memory>,
    image_fence: Option<B::Fence>,
}

impl<B: Backend> ImageState<B> {
    pub unsafe fn new(
        mut descriptor_set: DescriptorSet<B, Graphics>,
        device_state: &mut DeviceState<B, Graphics>,
        adapter_state: &AdapterState<B>,
        texture: Image,
        usage: buffer::Usage,
        staging_pool: &mut CommandPool<B, Graphics>,
    ) -> Self {
        const MIP_LEVELS: u8 = 1;

        let mut image_buffer_state = BufferState::new_texture(
            Rc::clone(&descriptor_set.layout.device_state),
            &device_state.device,
            adapter_state,
            &texture,
            usage,
        );

        let image_buffer = image_buffer_state.buffer.take().unwrap();

        let mut image = device_state
            .device
            .create_image(
                texture.get_kind().clone(),
                MIP_LEVELS,
                ColorFormat::SELF,
                i::Tiling::Optimal,
                i::Usage::TRANSFER_DST | i::Usage::SAMPLED,
                i::ViewCapabilities::empty(),
            )
            .unwrap();

        let device = &mut device_state.device;
        let requirements = device.get_image_requirements(&image);

        let device_memory_type =
            BufferState::<B, Graphics>::find_buffer_memory(&adapter_state.mem_types, &requirements, m::Properties::DEVICE_LOCAL);

        let device_image_memory = device.allocate_memory(device_memory_type, requirements.size).unwrap();

        device.bind_image_memory(&device_image_memory, 0, &mut image).unwrap();

        let image_view = device
            .create_image_view(&image, i::ViewKind::D2, ColorFormat::SELF, f::Swizzle::NO, COLOR_RANGE)
            .unwrap();

        let sampler = device
            .create_sampler(i::SamplerInfo::new(i::Filter::Linear, i::WrapMode::Clamp))
            .unwrap();

        descriptor_set.write_to_state(
            device,
            vec![
                DescriptorSetWrite {
                    binding: 0,
                    array_offset: 0,
                    descriptors: Some(pso::Descriptor::Image(&image_view, i::Layout::Undefined)),
                },
                DescriptorSetWrite {
                    binding: 1,
                    array_offset: 0,
                    descriptors: Some(pso::Descriptor::Sampler(&sampler)),
                },
            ],
        );

        let transferred_image_fence = device.create_fence(false).unwrap();

        {
            let row_alignment_mask = adapter_state.limits.min_buffer_copy_pitch_alignment as u32 - 1;
            let row_pitch = texture.row_pitch(row_alignment_mask);
            let (image_width, image_height) = texture.get_dimensions();

            // Copy the buffer to texture
            let mut cmd_buffer = staging_pool.acquire_command_buffer::<command::OneShot>();
            cmd_buffer.begin();

            let image_barrier = m::Barrier::Image {
                states: (i::Access::empty(), i::Layout::Undefined)..(i::Access::TRANSFER_WRITE, i::Layout::ShaderReadOnlyOptimal),
                target: &image,
                families: None,
                range: COLOR_RANGE.clone(),
            };

            cmd_buffer.pipeline_barrier(
                PipelineStage::TRANSFER..PipelineStage::FRAGMENT_SHADER,
                m::Dependencies::empty(),
                &[image_barrier],
            );

            cmd_buffer.copy_buffer_to_image(
                &image_buffer,
                &image,
                i::Layout::TransferDstOptimal,
                &[command::BufferImageCopy {
                    buffer_offset: 0,
                    buffer_width: row_pitch / RGBA_IMAGE_STRIDE as u32,
                    buffer_height: image_width,
                    image_layers: i::SubresourceLayers {
                        aspects: f::Aspects::COLOR,
                        level: 0,
                        layers: 0..1,
                    },
                    image_offset: i::Offset { x: 0, y: 0, z: 0 },
                    image_extent: i::Extent {
                        width: image_width,
                        height: image_height,
                        depth: 1,
                    },
                }],
            );

            let image_barrier = m::Barrier::Image {
                states: (i::Access::TRANSFER_WRITE, i::Layout::TransferDstOptimal)
                    ..(i::Access::SHADER_READ, i::Layout::ShaderReadOnlyOptimal),
                target: &image,
                families: None,
                range: COLOR_RANGE.clone(),
            };

            cmd_buffer.pipeline_barrier(
                PipelineStage::TRANSFER..PipelineStage::FRAGMENT_SHADER,
                m::Dependencies::empty(),
                &[image_barrier],
            );

            cmd_buffer.finish();

            device_state.queue_group.queues[0].submit_nosemaphores(std::iter::once(&cmd_buffer), Some(&transferred_image_fence));
        }

        ImageState {
            descriptor_set: descriptor_set,
            buffer: Some(image_buffer),
            image: Some(image),
            image_view: Some(image_view),
            sampler: Some(sampler),
            memory: Some(device_image_memory),
            image_fence: Some(transferred_image_fence),
        }
    }

    pub unsafe fn load_image_to_state(
        device_state: Rc<RefCell<DeviceState<B, Graphics>>>,
        adapter_state: &AdapterState<B>,
        image_file_path: &str,
        descriptor_set: DescriptorSetLayout<B, Graphics>,
    ) -> Self {
        let device_state_ref = device_state.as_ref().borrow();
        let device = &device_state_ref.device;

        let mut desc_pool = device
            .create_descriptor_pool(
                1,
                &[
                    pso::DescriptorRangeDesc {
                        ty: pso::DescriptorType::SampledImage,
                        count: 1,
                    },
                    pso::DescriptorRangeDesc {
                        ty: pso::DescriptorType::Sampler,
                        count: 1,
                    },
                ],
            )
            .ok();

        let descriptor_set = descriptor_set.create_set(desc_pool.as_mut().unwrap());

        let image_data = Loader::from_file(&image_file_path).expect("Failed to load image");

        let mut staging_pool = device
            .create_command_pool_typed(&device_state_ref.queue_group, pool::CommandPoolCreateFlags::empty())
            .unwrap();

        let image_state = ImageState::new(
            descriptor_set,
            &mut device_state.borrow_mut(),
            adapter_state,
            image_data,
            buffer::Usage::TRANSFER_SRC,
            &mut staging_pool,
        );

        image_state.wait_for_transfer();

        device.destroy_command_pool(staging_pool.into_raw());

        image_state
    }

    pub unsafe fn wait_for_transfer(&self) {
        let device = &self.descriptor_set.layout.device_state.as_ref().borrow().device;
        device.wait_for_fence(&self.image_fence.as_ref().unwrap(), !0).unwrap();
    }

    pub fn get_layout(&self) -> &B::DescriptorSetLayout {
        self.descriptor_set.get_layout()
    }
}

impl<B: Backend> Drop for ImageState<B> {
    fn drop(&mut self) {
        let device = self.descriptor_set.layout.device_state.as_ref();
        let device = &device.borrow().device;

        let image_fence = self.image_fence.take().unwrap();
        unsafe {
            device.wait_for_fence(&image_fence, !0).unwrap();
            device.destroy_fence(image_fence);

            device.destroy_sampler(self.sampler.take().unwrap());
            device.destroy_image_view(self.image_view.take().unwrap());
            device.destroy_image(self.image.take().unwrap());
            device.destroy_buffer(self.buffer.take().unwrap());

            device.free_memory(self.memory.take().unwrap());
        }
    }
}

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
