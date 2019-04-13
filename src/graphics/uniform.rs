use gfx_hal::*;
use graphics::buffer::BufferState;
use graphics::descriptor::{DescriptorSet, DescriptorSetWrite};
use graphics::device::DeviceState;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Uniform<B: Backend> {
    pub buffer_state: Option<BufferState<B, Graphics>>,
    pub descriptor_set: Option<DescriptorSet<B, Graphics>>,
}

impl<B: Backend> Uniform<B> {
    pub unsafe fn new<T>(
        device_state: Rc<RefCell<DeviceState<B, Graphics>>>,
        memory_types: &[MemoryType],
        uniform_buffer_source: &[T],
        mut descriptor_set: DescriptorSet<B, Graphics>,
        binding: u32,
    ) -> Self
    where
        T: Copy,
    {
        let buffer_state = BufferState::new(
            Rc::clone(&device_state),
            uniform_buffer_source,
            buffer::Usage::UNIFORM,
            memory_types,
        );

        descriptor_set.write_to_state(
            &mut device_state.as_ref().borrow_mut().device,
            vec![DescriptorSetWrite {
                binding,
                array_offset: 0,
                descriptors: Some(pso::Descriptor::Buffer(buffer_state.get_buffer(), None..None)),
            }],
        );

        Uniform {
            buffer_state: Some(buffer_state),
            descriptor_set: Some(descriptor_set),
        }
    }

    pub fn get_layout(&self) -> &B::DescriptorSetLayout {
        self.descriptor_set.as_ref().unwrap().get_layout()
    }
}
