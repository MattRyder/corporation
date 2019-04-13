use gfx_hal::memory as m;
use gfx_hal::*;
use graphics::adapter::AdapterState;
use graphics::device::DeviceState;
use graphics::image::Image;
use std::cell::RefCell;
use std::rc::Rc;

pub struct BufferState<B: Backend, C: Capability> {
  device_state: Rc<RefCell<DeviceState<B, C>>>,
  memory: Option<B::Memory>,
  pub buffer: Option<B::Buffer>,
  size: u64,
}

impl<B: Backend, C: Capability> BufferState<B, C> {
  pub unsafe fn new<T>(
    device_state: Rc<RefCell<DeviceState<B, C>>>,
    buffer_source: &[T],
    buffer_usage: buffer::Usage,
    memory_types: &[MemoryType],
  ) -> Self
  where
    T: Copy,
  {
    let stride = std::mem::size_of::<T>() as u64;
    let upload_size = buffer_source.len() as u64 * stride;

    let device = &device_state.borrow().device;

    let mut buffer = device.create_buffer(upload_size, buffer_usage).unwrap();

    let (memory, memory_size) = Self::allocate_buffer_memory(device, &buffer, memory_types, m::Properties::CPU_VISIBLE);

    device.bind_buffer_memory(&memory, 0, &mut buffer).unwrap();

    // Write the data to the buffer
    {
      let mut data_target = device.acquire_mapping_writer::<T>(&memory, 0..memory_size).unwrap();
      data_target[0..buffer_source.len()].copy_from_slice(&buffer_source);
      device.release_mapping_writer(data_target).unwrap();
    }

    BufferState {
      device_state: Rc::clone(&device_state),
      memory: Some(memory),
      buffer: Some(buffer),
      size: memory_size,
    }
  }

  /// Locates an available space in memory for the buffer depending on buffer requirements and memory property
  pub fn find_buffer_memory(memory_types: &[MemoryType], memory_requirements: &m::Requirements, property: m::Properties) -> MemoryTypeId {
    memory_types
      .iter()
      .enumerate()
      .position(|(id, mem_type)| memory_requirements.type_mask & (1 << id) != 0 && mem_type.properties.contains(property))
      .unwrap()
      .into()
  }

  unsafe fn allocate_buffer_memory(
    device: &B::Device,
    buffer: &B::Buffer,
    memory_types: &[MemoryType],
    memory_property: m::Properties,
  ) -> (B::Memory, u64) {
    let buffer_mem_requirements = device.get_buffer_requirements(&buffer);

    let size = buffer_mem_requirements.size;

    let upload_type = Self::find_buffer_memory(memory_types, &buffer_mem_requirements, memory_property);

    let memory = device.allocate_memory(upload_type, size).unwrap();

    (memory, size)
  }

  pub unsafe fn new_texture(
    device_state: Rc<RefCell<DeviceState<B, C>>>,
    device: &B::Device,
    adapter_state: &AdapterState<B>,
    image: &Image,
    buffer_usage: buffer::Usage,
  ) -> Self {
    let row_alignment_mask = adapter_state.limits.min_buffer_copy_pitch_alignment as u32 - 1;

    let upload_size = image.get_upload_size(row_alignment_mask);

    let mut buffer = device.create_buffer(upload_size, buffer_usage).unwrap();

    let (memory, memory_size) = Self::allocate_buffer_memory(device, &buffer, &adapter_state.mem_types, m::Properties::CPU_VISIBLE);

    device.bind_buffer_memory(&memory, 0, &mut buffer).unwrap();

    {
      let mut data_target = device.acquire_mapping_writer::<u8>(&memory, 0..memory_size).unwrap();

      let (_width, height) = image.get_dimensions();

      let img = &image.get_image().clone().into_raw();
      let row_pitch = image.row_pitch(row_alignment_mask);

      // Parse each row of the texture into the buffer:
      for y in 0..(height as usize) {
        let row_range = image.row_range(y);
        let row = &(img)[row_range.start..row_range.end];

        let dst_base = y * row_pitch as usize;
        data_target[dst_base..dst_base + row.len()].copy_from_slice(row);
      }

      device.release_mapping_writer(data_target).unwrap();
    }

    BufferState {
      device_state: Rc::clone(&device_state),
      memory: Some(memory),
      buffer: Some(buffer),
      size: upload_size,
    }
  }

  pub fn get_buffer(&self) -> &B::Buffer {
    self.buffer.as_ref().unwrap()
  }

  pub fn get_buffer_view(&self) -> buffer::IndexBufferView<B> {
    buffer::IndexBufferView {
      buffer: self.buffer.as_ref().unwrap(),
      offset: 0,
      index_type: IndexType::U16,
    }
  }
}
