use gfx_hal::format as f;
use gfx_hal::*;
use graphics::gfx_hal::device::DeviceState;
use graphics::gfx_hal::shader;
use mesh::vertex::Vertex;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PipelineState<B: Backend> {
    pub pipeline: Option<B::GraphicsPipeline>,
    pub pipeline_layout: Option<B::PipelineLayout>,
    device_state: Rc<RefCell<DeviceState<B, Graphics>>>,
}

impl<B: Backend> PipelineState<B> {
    pub unsafe fn new<IS>(descriptor_layouts: IS, render_pass: &B::RenderPass, device_state: Rc<RefCell<DeviceState<B, Graphics>>>) -> Self
    where
        IS: IntoIterator,
        IS::Item: std::borrow::Borrow<B::DescriptorSetLayout>,
    {
        let device = &device_state.as_ref().borrow().device;

        let pipeline_layout = device
            .create_pipeline_layout(descriptor_layouts, &[(pso::ShaderStageFlags::VERTEX, 0..8)])
            .unwrap();

        let vs_module = Self::create_shader_module(
            "VS_SHADER",
            shader::Kind::Vertex,
            concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/quad_150.vert"),
            Rc::clone(&device_state),
        );

        let fs_module = Self::create_shader_module(
            "FS_SHADER",
            shader::Kind::Fragment,
            concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/quad_150.frag"),
            Rc::clone(&device_state),
        );

        let pipeline = {
            const ENTRY_MAIN: &str = "main";

            let (vs_entry, fs_entry) = (
                pso::EntryPoint {
                    entry: ENTRY_MAIN,
                    module: &vs_module,
                    specialization: pso::Specialization {
                        constants: &[pso::SpecializationConstant { id: 0, range: 0..4 }],
                        data: std::mem::transmute::<&f32, &[u8; 4]>(&0.5f32),
                    },
                },
                pso::EntryPoint {
                    entry: ENTRY_MAIN,
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
                main_pass: render_pass,
            };

            let mut pipeline_description = pso::GraphicsPipelineDesc::new(
                shader_entries,
                Primitive::TriangleStrip,
                pso::Rasterizer::FILL,
                &pipeline_layout,
                subpass,
            );

            pipeline_description
                .blender
                .targets
                .push(pso::ColorBlendDesc(pso::ColorMask::ALL, pso::BlendState::ALPHA));

            pipeline_description.vertex_buffers.push(pso::VertexBufferDesc {
                binding: 0,
                stride: std::mem::size_of::<Vertex>() as u32,
                rate: 0,
            });

            pipeline_description.attributes.push(pso::AttributeDesc {
                location: 0,
                binding: 0,
                element: pso::Element {
                    format: f::Format::Rgb32Float,
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

            device.create_graphics_pipeline(&pipeline_description, None).unwrap()
        };

        // Cleanup Shader module resources after use:
        device.destroy_shader_module(vs_module);
        device.destroy_shader_module(fs_module);

        PipelineState {
            pipeline: Some(pipeline),
            pipeline_layout: Some(pipeline_layout),
            device_state: Rc::clone(&device_state),
        }
    }

    unsafe fn create_shader_module(
        name: &str,
        kind: shader::Kind,
        file_path: &str,
        device_state: Rc<RefCell<DeviceState<B, Graphics>>>,
    ) -> B::ShaderModule {
        let shader_src = std::fs::read_to_string(file_path).expect(&format!("Failed to read shader from file: {}", name));

        let spirv_bin = shader::Loader::compile(name, &kind, &shader_src).expect(&format!("Failed to compile shader: {}", name));

        device_state.as_ref().borrow().device.create_shader_module(&spirv_bin).unwrap()
    }
}

impl<B: Backend> Drop for PipelineState<B> {
    fn drop(&mut self) {
        let device = &self.device_state.as_ref().borrow().device;
        unsafe {
            device.destroy_graphics_pipeline(self.pipeline.take().unwrap());
            device.destroy_pipeline_layout(self.pipeline_layout.take().unwrap());
        }
    }
}
