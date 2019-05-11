use super::{Image, ImageResourceState, RGBA_IMAGE_STRIDE};
use gfx_hal::format as f;
use gfx_hal::format::AsFormat;
use gfx_hal::image as i;
use gfx_hal::memory as m;
use gfx_hal::pso::PipelineStage;
use gfx_hal::Backend;
use gfx_hal::*;
use graphics::gfx_hal::adapter::AdapterState;
use graphics::gfx_hal::backend::ColorFormat;
use graphics::gfx_hal::buffer::BufferState;
use graphics::gfx_hal::descriptor::{DescriptorSet, DescriptorSetWrite};
use graphics::gfx_hal::device::DeviceState;
use std::rc::Rc;

pub const COLOR_RANGE: i::SubresourceRange = i::SubresourceRange {
    aspects: f::Aspects::COLOR,
    levels: 0..1,
    layers: 0..1,
};

pub struct TextureImageState<B: Backend> {
    pub image_resource_state: Option<ImageResourceState<B>>,
    pub descriptor_set: DescriptorSet<B, Graphics>,
    sampler: Option<B::Sampler>,
    buffer_state: BufferState<B, Graphics>,
    image_fence: Option<B::Fence>,
}

impl<B: Backend> TextureImageState<B> {
    pub unsafe fn new(
        mut descriptor_set: DescriptorSet<B, Graphics>,
        device_state: &mut DeviceState<B, Graphics>,
        adapter_state: &AdapterState<B>,
        texture: Image,
        usage: buffer::Usage,
        staging_pool: &mut CommandPool<B, Graphics>,
    ) -> Self {
        const MIP_LEVELS: u8 = 1;

        let image_buffer_state = BufferState::new_texture(
            Rc::clone(&descriptor_set.layout.device_state),
            &device_state.device,
            adapter_state,
            &texture,
            usage,
        );

        let mut image_state = ImageResourceState::new(
            device_state,
            adapter_state,
            COLOR_RANGE,
            Rc::clone(&descriptor_set.layout.device_state),
            texture.get_kind(),
        );

        let device = &mut device_state.device;

        let sampler = device
            .create_sampler(i::SamplerInfo::new(i::Filter::Linear, i::WrapMode::Clamp))
            .unwrap();

        descriptor_set.write_to_state(
            device,
            vec![
                DescriptorSetWrite {
                    binding: 0,
                    array_offset: 0,
                    descriptors: Some(pso::Descriptor::Image(image_state.image_view.as_ref().unwrap(), i::Layout::Undefined)),
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
                target: image_state.image.as_ref().unwrap(),
                families: None,
                range: COLOR_RANGE.clone(),
            };

            cmd_buffer.pipeline_barrier(
                PipelineStage::TRANSFER..PipelineStage::FRAGMENT_SHADER,
                m::Dependencies::empty(),
                &[image_barrier],
            );

            cmd_buffer.copy_buffer_to_image(
                &image_buffer_state.get_buffer(),
                &image_state.image.as_ref().unwrap(),
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
                target: image_state.image.as_ref().unwrap(),
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

        TextureImageState {
            buffer_state: image_buffer_state,
            descriptor_set,
            image_fence: Some(transferred_image_fence),
            image_resource_state: Some(image_state),
            sampler: Some(sampler),
        }
    }

    pub unsafe fn wait_for_transfer(&self) {
        let device = &self.descriptor_set.layout.device_state.as_ref().borrow().device;
        device.wait_for_fence(&self.image_fence.as_ref().unwrap(), !0).unwrap();
    }

    pub fn get_layout(&self) -> &B::DescriptorSetLayout {
        self.descriptor_set.get_layout()
    }
}

impl<B: Backend> Drop for TextureImageState<B> {
    fn drop(&mut self) {
        let device = self.descriptor_set.layout.device_state.as_ref();
        let device = &device.borrow().device;

        let image_fence = self.image_fence.take().unwrap();
        unsafe {
            device.wait_for_fence(&image_fence, !0).unwrap();
            device.destroy_fence(image_fence);

            device.destroy_sampler(self.sampler.take().unwrap());
        }
    }
}
