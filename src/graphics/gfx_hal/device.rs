use gfx_hal::*;

pub struct DeviceState<B: Backend, C: Capability> {
  pub device: B::Device,
  pub phys_device: B::PhysicalDevice,
  pub queue_group: QueueGroup<B, C>,
}

impl<B: Backend, C: Capability> DeviceState<B, C> {
  pub fn new(adapter: Adapter<B>, surface: &B::Surface) -> Self {
    let (device, queue_group) = adapter
      .open_with::<_, C>(1, |family| surface.supports_queue_family(family))
      .unwrap();

    DeviceState {
      device,
      queue_group,
      phys_device: adapter.physical_device,
    }
  }

  /// Create a command pool from the provided queue type capability
  pub unsafe fn create_command_pool(&self) -> CommandPool<B, C>
  where
    B: Backend,
    C: Capability,
  {
    self
      .device
      .create_command_pool_typed(&self.queue_group, pool::CommandPoolCreateFlags::empty())
      .unwrap()
  }
}
