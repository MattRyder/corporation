use gfx_backend;
use gfx_hal::buffer;
use gfx_hal::format::{AsFormat, ChannelType, Rgba8Srgb, Swizzle};
use gfx_hal::pool::CommandPoolCreateFlags;
use gfx_hal::pso;
use gfx_hal::pso::{PipelineStage, ShaderStageFlags};
use gfx_hal::queue::Submission;
use gfx_hal::{
    command, format as f, image as i, memory as m, pass,
    Backbuffer, DescriptorPool, Device, FrameSync, Graphics, Instance,
    IndexType, Primitive, PhysicalDevice, Surface, Swapchain, SwapchainConfig,
    window,
};
use graphics::{shader, texture, Vertex};

pub struct Builder;

pub type ColorFormat = Rgba8Srgb;

const COLOR_RANGE: i::SubresourceRange = i::SubresourceRange {
    aspects: f::Aspects::COLOR,
    levels: 0..1,
    layers: 0..1,
};

const CLEAR_COLOR: [f32; 4] = [0.255, 0.412, 0.882, 1.0];

const QUAD: [Vertex; 4] = [
    Vertex {
        a_Position: [-1.0, -1.0, 0.0],
        a_TexCoord: [0.0, 0.0],
    },
    Vertex {
        a_Position: [-1.0, 1.0, 0.0],
        a_TexCoord: [0.0, 1.0],
    },
    Vertex {
        a_Position: [1.0, 1.0, 0.0],
        a_TexCoord: [1.0, 1.0],
    },
    Vertex {
        a_Position: [1.0, -1.0, 0.0],
        a_TexCoord: [1.0, 0.0],
    },
];

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

impl Builder {

    #[cfg(any(
        feature = "vulkan",
        feature = "dx12",
        feature = "metal",
        feature = "gl"))]
    pub fn build(title: &str, width: f64, height: f64) {
        let mut events_loop = winit::EventsLoop::new();

        let window_builder = winit::WindowBuilder::new()
            .with_dimensions(winit::dpi::LogicalSize::new(
                width, height
            )).with_title(title);

        // Create backend
        #[cfg(not(feature = "gl"))]
        let (_window, _instance, mut adapters, mut surface) = {
            let window = window_builder.build(&events_loop).unwrap();
            let instance = gfx_backend::Instance::create("corporation", 1);
            let surface = instance.create_surface(&window);
            let adapters = instance.enumerate_adapters();
            (window, instance, adapters, surface)
        };
        #[cfg(feature = "gl")]
        let (mut adapters, mut surface) = {
            let window = {
                let builder = gfx_backend::config_context(
                    gfx_backend::glutin::ContextBuilder::new(),
                    ColorFormat::SELF,
                    None
                ).with_vsync(true);

                gfx_backend::glutin::GlWindow::new(window_builder, builder, &events_loop).unwrap()
            };

            let surface = gfx_backend::Surface::from_window(window);
            let adapters = surface.enumerate_adapters();
            (adapters, surface)
        };

        for adapter in &adapters {
            info!("Available {:?}", adapter.info);
        }

        let mut adapter = adapters.remove(0);
        let memory_types = adapter.physical_device.memory_properties().memory_types;
        let limits = adapter.physical_device.limits();

        info!("Adapter(0) {:?}", limits);

        let (device, mut queue_group) = adapter
            .open_with::<_, Graphics>(1, |family| surface.supports_queue_family(family))
            .expect("Failed to create device!");

        let mut command_pool = unsafe {
            device.create_command_pool_typed(&queue_group, CommandPoolCreateFlags::empty())
        }.expect("Failed to create command pool!");

        // Setup Render Pass
        let set_layout = unsafe {
            device.create_descriptor_set_layout(
                &[
                    pso::DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: pso::DescriptorType::SampledImage,
                        count: 1,
                        stage_flags: ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                    pso::DescriptorSetLayoutBinding {
                        binding: 1,
                        ty: pso::DescriptorType::Sampler,
                        count: 1,
                        stage_flags: ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                ],
                &[],
            )
        }.expect("Failed to create descriptor set layout!");

        let mut descriptor_pool = unsafe {
            device.create_descriptor_pool(
                1, // Number of sets
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
        }.expect("Failed to create descriptor pool!");
        let descriptor_set = unsafe {
            descriptor_pool.allocate_set(&set_layout)
        }.unwrap();

        // Buffer Allocations
        info!("Memory Types: {:?}", &memory_types);

        let buffer_stride = std::mem::size_of::<Vertex>() as u64;
        let buffer_len = QUAD.len() as u64 * buffer_stride;

        let mut vertex_buffer = unsafe {
            device.create_buffer(buffer_len, buffer::Usage::VERTEX)
        }.unwrap();

        // Calculate how much this buffer will take up on the device:
        let buffer_requirements = unsafe {
            device.get_buffer_requirements(&vertex_buffer)
        };

        // Find a memory region that is CPU visible:
        let cpu_visible_memory_type = memory_types
            .iter()
            .enumerate()
            .position(|(id, mem_type)| {
                buffer_requirements.type_mask & (1 << id) != 0 && mem_type.properties.contains(m::Properties::CPU_VISIBLE)
            })
            .unwrap()
            .into();

        let buffer_memory = unsafe {
            device.allocate_memory(cpu_visible_memory_type, buffer_requirements.size)
        }.unwrap();

        unsafe {
            device.bind_buffer_memory(&buffer_memory, 0, &mut vertex_buffer)
        }.expect("Failed to bind vertex buffer memory!");

        unsafe {
            let mut vertices = device
                .acquire_mapping_writer::<Vertex>(&buffer_memory, 0..buffer_requirements.size)
                .unwrap();

            vertices[0..QUAD.len()].copy_from_slice(&QUAD);
            device.release_mapping_writer(vertices).unwrap();
        }

        // Load the index buffer:
        let index_stride = std::mem::size_of::<u16>() as u64;
        let index_buffer_len = QUAD_INDICES.len() as u64 * index_stride;

        let mut index_buffer = unsafe {
            device.create_buffer(index_buffer_len, buffer::Usage::INDEX)
        }.unwrap();

        let index_buffer_reqs = unsafe { device.get_buffer_requirements(&index_buffer) };
        let index_buffer_memory = unsafe {
            device.allocate_memory(cpu_visible_memory_type, index_buffer_reqs.size)
        }.unwrap();
        unsafe {
            device.bind_buffer_memory(&index_buffer_memory, 0, &mut index_buffer)
        }.unwrap();

        unsafe {
            let mut indices = device
                .acquire_mapping_writer::<u16>(&index_buffer_memory, 0..index_buffer_reqs.size)
                .unwrap();

            indices[0..QUAD_INDICES.len()].copy_from_slice(&QUAD_INDICES);
            device.release_mapping_writer(indices).unwrap();
        }

        // Load an image:
        let texture = texture::Loader::from_file("resources/uv_grid.jpg").unwrap();
        let row_alignment_mask = limits.min_buffer_copy_pitch_alignment as u32 - 1;
        let row_pitch = texture.row_pitch(row_alignment_mask);
        let image_upload_size = texture.get_upload_size(row_alignment_mask);

        let mut image_buffer = unsafe {
            device.create_buffer(image_upload_size, buffer::Usage::TRANSFER_SRC)
        }.unwrap();
        let image_memory_reqs = unsafe { device.get_buffer_requirements(&image_buffer) };
        let image_buffer_memory = unsafe {
            device.allocate_memory(cpu_visible_memory_type, image_memory_reqs.size)
        }.unwrap();
        unsafe {
            device.bind_buffer_memory(&image_buffer_memory, 0, &mut image_buffer)
        }.expect("Failed to bind image memory!");

        unsafe {
            let mut data = device
                .acquire_mapping_writer::<u8>(&image_buffer_memory, 0..image_memory_reqs.size)
                .unwrap();
            let (_, height) = texture.get_dimensions();
            let img = &texture.get_image().clone().into_raw();

            // Parse each row of the texture into the buffer:
            for y in 0..(height as usize) {
                let row_range = texture.row_range(y);
                let row = &(img)[row_range.start..row_range.end];

                let dst_base = y * row_pitch as usize;
                data[dst_base..dst_base + row.len()].copy_from_slice(row);
            }

            device.release_mapping_writer(data).unwrap();
        }

        let mut image_grid = unsafe {
            device.create_image(
                texture.get_kind().clone(),
                1,
                ColorFormat::SELF,
                i::Tiling::Optimal,
                i::Usage::TRANSFER_DST | i::Usage::SAMPLED,
                i::ViewCapabilities::empty(),
            )
        }.unwrap();
        let image_req = unsafe { device.get_image_requirements(&image_grid) };

        // find some device-local memory to place the image:
        let device_type = memory_types
            .iter()
            .enumerate()
            .position(|(id, mem_type)|
                image_req.type_mask & (1 << id) != 0 && mem_type.properties.contains(m::Properties::DEVICE_LOCAL))
            .unwrap()
            .into();
        let image_memory = unsafe {
            device.allocate_memory(device_type, image_req.size)
        }.unwrap();

        unsafe {
            device.bind_image_memory(&image_memory, 0, &mut image_grid)
        }.expect("Failed to bind image memory!");

        let image_view = unsafe {
            device.create_image_view(&image_grid, i::ViewKind::D2, ColorFormat::SELF, Swizzle::NO, COLOR_RANGE.clone())
        }.unwrap();

        let sampler = unsafe {
            device.create_sampler(i::SamplerInfo::new(i::Filter::Linear, i::WrapMode::Clamp))
        }.expect("Failed to create sampler!");

        unsafe {
            device.write_descriptor_sets(vec![
                pso::DescriptorSetWrite {
                    set: &descriptor_set,
                    binding: 0,
                    array_offset: 0,
                    descriptors: Some(pso::Descriptor::Image(&image_view, i::Layout::Undefined)),
                },
                pso::DescriptorSetWrite {
                    set: &descriptor_set,
                    binding: 1,
                    array_offset: 0,
                    descriptors: Some(pso::Descriptor::Sampler(&sampler)),
                },
            ]);
        }

        let mut frame_semaphore = device.create_semaphore().expect("Failed to create frame semaphore!");
        let mut frame_fence = device.create_fence(false).expect("Failed to create frame fence!");

        unsafe {
            let mut cmd_buffer = command_pool.acquire_command_buffer::<command::OneShot>();
            let (width, height) = texture.get_dimensions();

            cmd_buffer.begin();

            let image_barrier = m::Barrier::Image {
                states: (i::Access::empty(), i::Layout::Undefined)..(i::Access::TRANSFER_WRITE, i::Layout::TransferDstOptimal),
                target: &image_grid,
                families: None,
                range: COLOR_RANGE.clone(),
            };

            cmd_buffer.pipeline_barrier(
                PipelineStage::TOP_OF_PIPE..PipelineStage::TRANSFER,
                m::Dependencies::empty(),
                &[image_barrier],
            );

            cmd_buffer.copy_buffer_to_image(
                &image_buffer,
                &image_grid,
                i::Layout::TransferDstOptimal,
                &[command::BufferImageCopy {
                    buffer_offset: 0,
                    buffer_width: row_pitch / (texture::RGBA_IMAGE_STRIDE as u32),
                    buffer_height: width,
                    image_layers: i::SubresourceLayers {
                        aspects: f::Aspects::COLOR,
                        level: 0,
                        layers: 0..1,
                    },
                    image_offset: i::Offset { x: 0, y: 0, z: 0 },
                    image_extent: i::Extent { width, height, depth: 1 },
                }],
            );

            let image_barrier = m::Barrier::Image {
                states: (i::Access::TRANSFER_WRITE, i::Layout::TransferDstOptimal)
                    ..(i::Access::SHADER_READ, i::Layout::ShaderReadOnlyOptimal),
                target: &image_grid,
                families: None,
                range: COLOR_RANGE.clone(),
            };
            cmd_buffer.pipeline_barrier(
                PipelineStage::TRANSFER..PipelineStage::FRAGMENT_SHADER,
                m::Dependencies::empty(),
                &[image_barrier],
            );

            cmd_buffer.finish();

            queue_group.queues[0].submit_nosemaphores(Some(&cmd_buffer), Some(&mut frame_fence));

            device.wait_for_fence(&frame_fence, !0).expect("Failed to wait for frame fence!");
        }

        // Sort out swap chain, start by grabbing surface format:
        let (caps, formats, _present_modes, _composite_alphas) =
            surface.compatibility(&mut adapter.physical_device);

        info!("Formats: {:?}", &formats);

        let format = formats.map_or(ColorFormat::SELF, |formats| {
            formats
                .iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .map(|format| *format)
                .unwrap_or(formats[0])
        });

        let default_extent = window::Extent2D { width: width as u32, height: height as u32 };
        let swap_config = SwapchainConfig::from_caps(&caps, format, default_extent);
        let frame_extent = swap_config.extent.to_extent();
        info!("Extent: {:?}", &frame_extent);
        info!("Swap config: {:?}", &swap_config);

        let (mut swap_chain, mut backbuffer) = unsafe {
            device.create_swapchain(&mut surface, swap_config, None)
        }.expect("Failed to create swapchain!");

        // Create render pass:
        let render_pass = {
            let attachment = pass::Attachment {
                format: Some(format),
                samples: 1,
                ops: pass::AttachmentOps::new(pass::AttachmentLoadOp::Clear, pass::AttachmentStoreOp::Store),
                stencil_ops: pass::AttachmentOps::DONT_CARE,
                layouts: i::Layout::Undefined..i::Layout::Present,
            };

            let subpass = pass::SubpassDesc {
                colors: &[(0, i::Layout::ColorAttachmentOptimal)],
                depth_stencil: None,
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };

            let dependency = pass::SubpassDependency {
                passes: pass::SubpassRef::External..pass::SubpassRef::Pass(0),
                stages: PipelineStage::COLOR_ATTACHMENT_OUTPUT..PipelineStage::COLOR_ATTACHMENT_OUTPUT,
                accesses: i::Access::empty()..(i::Access::COLOR_ATTACHMENT_READ | i::Access::COLOR_ATTACHMENT_WRITE),
            };

            unsafe {
                device.create_render_pass(&[attachment], &[subpass], &[dependency])
            }.expect("Failed to create render pass")
        };

        info!("Created Render Pass: {:?}", &render_pass);

        let (mut frame_images, mut framebuffers) = match backbuffer {
            Backbuffer::Images(images) => {
                let pairs = images
                    .into_iter()
                    .map(|image| {
                        let rtv = unsafe {
                            device.create_image_view(&image, i::ViewKind::D2, format, Swizzle::NO, COLOR_RANGE.clone())
                        }.unwrap();

                        (image, rtv)
                    })
                    .collect::<Vec<_>>();

                let fbos = pairs
                    .iter()
                    .map(|&(_, ref rtv)| unsafe {
                        device.create_framebuffer(&render_pass, Some(rtv), frame_extent)
                    }.unwrap())
                    .collect();

                (pairs, fbos)
            }
            Backbuffer::Framebuffer(fbo) => (Vec::new(), vec![fbo]),
        };

        let pipeline_layout = unsafe {
            device.create_pipeline_layout(std::iter::once(&set_layout), &[(pso::ShaderStageFlags::VERTEX, 0..8)])
        }.expect("Failed to create pipeline layout!");

        let pipeline = {
            const ENTRY_NAME: &str = "main";

            let vs_module = {
                let vs_shader_src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/quad_150.vert"))
                    .expect("Failed to read quad shader vs");

                let vs_spirv =
                    shader::Loader::compile("VS_SHADER", shader::Kind::Vertex, &vs_shader_src).expect("Failed to compile quad shader vs");

                unsafe { device.create_shader_module(&vs_spirv) }.unwrap()
            };

            let fs_module = {
                let fs_shader_src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/quad_150.frag"))
                    .expect("Failed to read quad shader fs");

                let fs_spirv =
                    shader::Loader::compile("FS_SHADER", shader::Kind::Fragment, &fs_shader_src).expect("Failed to compile quad shader fs");

                unsafe { device.create_shader_module(&fs_spirv) }.unwrap()
            };

            let pipeline = {
                let (vs_entry, fs_entry) = (
                    pso::EntryPoint::<gfx_backend::Backend> {
                        entry: ENTRY_NAME,
                        module: &vs_module,
                        specialization: pso::Specialization {
                            // Set the `scale` constant in the shader module:
                            constants: &[pso::SpecializationConstant { id: 0, range: 0..4 }],
                            data: unsafe { std::mem::transmute::<&f32, &[u8; 4]>(&0.8f32) },
                        },
                    },
                    pso::EntryPoint::<gfx_backend::Backend> {
                        entry: ENTRY_NAME,
                        module: &fs_module,
                        specialization: pso::Specialization::default(),
                    },
                );

                let shader_entries = pso::GraphicsShaderSet {
                    vertex: vs_entry,
                    hull: None,
                    domain: None,
                    geometry: None,
                    fragment: Some(fs_entry),
                };

                let subpass = pass::Subpass {
                    index: 0,
                    main_pass: &render_pass,
                };

                let mut pipeline_description = pso::GraphicsPipelineDesc::new(
                    shader_entries,
                    Primitive::TriangleStrip,
                    pso::Rasterizer::FILL,
                    &pipeline_layout,
                    subpass,
                );

                pipeline_description.blender.targets.push(pso::ColorBlendDesc(
                    pso::ColorMask::ALL,
                    pso::BlendState::ALPHA,
                ));

                pipeline_description.vertex_buffers.push(pso::VertexBufferDesc {
                    binding: 0,
                    stride: std::mem::size_of::<Vertex>() as u32,
                    rate: 0,
                });

                pipeline_description.attributes.push(pso::AttributeDesc {
                    location: 0,
                    binding: 0,
                    element: pso::Element {
                        format: f::Format::Rg32Float,
                        offset: 0,
                    },
                });
                pipeline_description.attributes.push(pso::AttributeDesc {
                    location: 1,
                    binding: 0,
                    element: pso::Element {
                        format: f::Format::Rg32Float,
                        offset: 12,
                    },
                });

                unsafe { device.create_graphics_pipeline(&pipeline_description, None) }
            };

            // Cleanup Shader module resources after use:
            unsafe { device.destroy_shader_module(vs_module) };
            unsafe { device.destroy_shader_module(fs_module) };

            pipeline.unwrap()
        };

        // Setup Viewport:
        let mut viewport = pso::Viewport {
            rect: pso::Rect {
                x: 0,
                y: 0,
                w: frame_extent.width as _,
                h: frame_extent.height as _,
            },
            depth: 0.0..1.0,
        };

        let mut engine_running = true;
        let mut recreate_swapchain = false;
        let mut resize_dimensions = window::Extent2D {
            width: 0, height: 0,
        };

        while engine_running {
            events_loop.poll_events(|event| {
                if let winit::Event::WindowEvent { event, .. } = event {
                    match event {
                        // Handle the window being closed:
                        winit::WindowEvent::CloseRequested => engine_running = false,

                        // Handle the window being resized:
                        winit::WindowEvent::Resized(dimensions) => {
                            info!("Window Resized: {:?}", dimensions);

                            #[cfg(feature = "gl")]
                            surface.get_window().resize(
                                dimensions.to_physical(surface.get_window().get_hidpi_factor()));

                            recreate_swapchain = true;
                            resize_dimensions.width = dimensions.width as u32;
                            resize_dimensions.height = dimensions.height as u32;
                        },
                        _ => {}
                    }
                }
            });

            // Handle if the swapchain requires rebuilding at runtime (i.e. resized)
            if recreate_swapchain {
                device.wait_idle().unwrap();

                let (caps, formats, _present_modes, _composite_alphas) =
                    surface.compatibility(&mut adapter.physical_device);

                // verify that the format we're using still exists:
                assert!(formats.iter().any(|fs| fs.contains(&format)));

                let swap_config = SwapchainConfig::from_caps(&caps, format, resize_dimensions);
                info!("Recreation of swapchain, config: {:?}", &swap_config);

                let extent = swap_config.extent.to_extent();

                let (new_swap_chain, new_backbuffer) = unsafe {
                    device.create_swapchain(&mut surface, swap_config, Some(swap_chain))
                }.expect("Failed to recreate swap chain!");

                for framebuffer in framebuffers {
                    unsafe { device.destroy_framebuffer(framebuffer) };
                }
                for (_, rtv) in frame_images {
                    unsafe { device.destroy_image_view(rtv) };
                }

                backbuffer = new_backbuffer;
                swap_chain = new_swap_chain;

                let (new_frame_images, new_framebuffers) = match backbuffer {
                    Backbuffer::Images(images) => {
                        let pairs = images
                            .into_iter()
                            .map(|image| {
                                let rtv = unsafe {
                                    device.create_image_view(&image, i::ViewKind::D2, format, Swizzle::NO, COLOR_RANGE.clone())
                                }.unwrap();

                                (image, rtv)
                            })
                            .collect::<Vec<_>>();

                        let fbos = pairs
                            .iter()
                            .map(|&(_, ref rtv)| unsafe {
                                device.create_framebuffer(&render_pass, Some(rtv), frame_extent)
                            }.unwrap())
                            .collect();

                        (pairs, fbos)
                    }
                    Backbuffer::Framebuffer(fbo) => (Vec::new(), vec![fbo]),
                };

                framebuffers = new_framebuffers;
                frame_images = new_frame_images;

                viewport.rect.w = extent.width as _;
                viewport.rect.h = extent.height as _;

                recreate_swapchain = false;
            }

            unsafe { device.reset_fence(&frame_fence) }.unwrap();
            unsafe { command_pool.reset() };

            let frame: gfx_hal::SwapImageIndex = unsafe {
                match swap_chain.acquire_image(!0, FrameSync::Semaphore(&mut frame_semaphore)) {
                    Ok(i) => i,
                    Err(_) => {
                        recreate_swapchain = true;
                        continue;
                    }
                }
            };

            let index_buffer_view = buffer::IndexBufferView {
                buffer: &index_buffer,
                offset: 0,
                index_type: IndexType::U16,
            };

            // Start performing the rendering:
            let mut cmd_buffer = command_pool.acquire_command_buffer::<command::OneShot>();

            unsafe {
                cmd_buffer.begin();

                cmd_buffer.set_viewports(0, &[viewport.clone()]);
                cmd_buffer.set_scissors(0, &[viewport.rect]);
                cmd_buffer.bind_graphics_pipeline(&pipeline);
                cmd_buffer.bind_vertex_buffers(0, Some((&vertex_buffer, 0)));
                cmd_buffer.bind_index_buffer(index_buffer_view);
                cmd_buffer.bind_graphics_descriptor_sets(&pipeline_layout, 0, Some(&descriptor_set), &[]);

                {
                    let mut encoder = cmd_buffer.begin_render_pass_inline(
                        &render_pass,
                        &framebuffers[frame as usize],
                        viewport.rect,
                        &[command::ClearValue::Color(command::ClearColor::Float(CLEAR_COLOR.clone()))]
                    );

                    encoder.draw_indexed(0..6, 0, 0..1);
                }

                cmd_buffer.finish();


                let submission = Submission {
                    command_buffers: Some(&cmd_buffer),
                    wait_semaphores: Some((&frame_semaphore, PipelineStage::BOTTOM_OF_PIPE)),
                    signal_semaphores: &[],
                };
                queue_group.queues[0].submit(submission, Some(&mut frame_fence));

                device.wait_for_fence(&frame_fence, !0).expect("Failed to wait for fence!");

                // Now display that frame, rebuild on error:
                if let Err(_) = swap_chain.present_nosemaphores(&mut queue_group.queues[0], frame) {
                    recreate_swapchain = true;
                }
            }
        }

        // Shutting down, clean up allocated resources:
        device.wait_idle().unwrap();
        unsafe {
            device.destroy_command_pool(command_pool.into_raw());
            device.destroy_descriptor_pool(descriptor_pool);
            device.destroy_descriptor_set_layout(set_layout);

            device.destroy_buffer(vertex_buffer);
            device.destroy_buffer(image_buffer);
            device.destroy_image(image_grid);
            device.destroy_image_view(image_view);
            device.destroy_sampler(sampler);
            device.destroy_fence(frame_fence);
            device.destroy_semaphore(frame_semaphore);
            device.destroy_render_pass(render_pass);

            device.free_memory(buffer_memory);
            device.free_memory(image_memory);
            device.free_memory(image_buffer_memory);

            device.destroy_graphics_pipeline(pipeline);
            device.destroy_pipeline_layout(pipeline_layout);

            for framebuffer in framebuffers {
                device.destroy_framebuffer(framebuffer);
            }

            for (_, rtv) in frame_images {
                device.destroy_image_view(rtv);
            }

            device.destroy_swapchain(swap_chain);
        }


    }

    #[cfg(not(any(feature = "vulkan", feature = "dx12", feature = "metal", feature = "gl")))]
    pub fn build(title: &str, width: f64, height: f64) {
        error!("Cannot find implemented feature for gfx-hal");
    }
}