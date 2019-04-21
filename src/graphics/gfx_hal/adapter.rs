use gfx_hal::*;

/// Defines the capabilities of a single graphics adapter
pub struct AdapterState<B: Backend> {
  pub adapter: Option<Adapter<B>>,
  pub mem_types: Vec<MemoryType>,
  pub limits: Limits,
}

impl<B: Backend> AdapterState<B> {
  pub fn new(adapters: &mut Vec<Adapter<B>>) -> Self {
    for adapter in adapters.iter() {
      info!("Available {:?}", adapter.info);
    }

    // TODO: Iterate and choose the most optimal adapter
    warn!("Choosing Adapter(0) by default.");

    let adapter = adapters.remove(0);

    let mem_types = adapter.physical_device.memory_properties().memory_types;
    let limits = adapter.physical_device.limits();

    info!("Adapter Limits: {:?}", &limits);

    AdapterState {
      adapter: Some(adapter),
      mem_types,
      limits,
    }
  }
}
