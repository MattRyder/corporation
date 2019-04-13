use gfx_hal::*;
use graphics::device::DeviceState;
use std::cell::RefCell;
use std::rc::Rc;
use std::borrow::Borrow;

pub struct DescriptorSetLayout<B: Backend, C: Capability> {
  pub device_state: Rc<RefCell<DeviceState<B, C>>>,
  layout: Option<B::DescriptorSetLayout>,
}

pub struct DescriptorSet<B: Backend, C: Capability> {
  pub layout: DescriptorSetLayout<B, C>,
  pub set: Option<B::DescriptorSet>,
}

pub struct DescriptorSetWrite<W> {
  pub binding: pso::DescriptorBinding,
  pub array_offset: pso::DescriptorArrayIndex,
  pub descriptors: W,
}

impl<B: Backend, C: Capability> DescriptorSetLayout<B, C> {
  pub unsafe fn new(device_state: Rc<RefCell<DeviceState<B, C>>>, bindings: Vec<pso::DescriptorSetLayoutBinding>) -> Self {
    let layout = device_state.as_ref().borrow().device.create_descriptor_set_layout(&bindings, &[]).ok();

    DescriptorSetLayout { device_state, layout }
  }

  pub unsafe fn create_set(self, pool: &mut B::DescriptorPool) -> DescriptorSet<B, C> {
    let set = pool.allocate_set(self.layout.as_ref().unwrap()).unwrap();

    DescriptorSet {
      set: Some(set),
      layout: self,
    }
  }
}

impl<B: Backend, C: Capability> Drop for DescriptorSetLayout<B, C> {
  fn drop(&mut self) {
    // unsafe {
    //   device.destroy_descriptor_set_layout(self.layout.take().unwrap());
    // }
  }
}

impl<B: Backend, C: Capability> DescriptorSet<B, C> {
  pub fn write_to_state<'a, 'b: 'a, W>(&'b mut self, device: &mut B::Device, write_sets: Vec<DescriptorSetWrite<W>>)
  where
    W: IntoIterator,
    W::Item: std::borrow::Borrow<pso::Descriptor<'a, B>>,
  {
    let set = self.set.as_ref().unwrap();
    let writable_sets: Vec<_> = write_sets
      .into_iter()
      .map(|d| pso::DescriptorSetWrite {
        binding: d.binding,
        array_offset: d.array_offset,
        descriptors: d.descriptors,
        set,
      })
      .collect();

    unsafe { device.write_descriptor_sets(writable_sets); }
  }

  pub fn get_layout(&self) -> &B::DescriptorSetLayout {
    self.layout.layout.borrow().as_ref().unwrap()
  }
}
