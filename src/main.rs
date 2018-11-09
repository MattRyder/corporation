#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate image;

use gfx::handle::Sampler;
use gfx::handle::ShaderResourceView;
use gfx::traits::FactoryExt;
use glutin::Api::OpenGl;

pub mod errors;
pub mod graphics;

use errors::*;
use graphics::{ColorFormat, DepthFormat, GraphicsContext, Pipeline, Vertex, TextureLoader};

const WINDOW_WIDTH: f64 = 640.0;
const WINDOW_HEIGHT: f64 = 480.0;
const WINDOW_TITLE: &str = "Corporation";

const CLEAR_COLOR: [f32; 4] = [0.55, 0.52, 1.0, 1.0];

const SQUARE_VERTS: [Vertex; 4] = [
    Vertex {
        position: [-1.0, -1.0, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        tex_coord: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        tex_coord: [1.0, 0.0],
    },
];

const SQUARE_INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

fn create_gl_window(
    title: &str,
    width: f64,
    height: f64,
) -> Result<(
    glutin::EventsLoop,
    glutin::GlWindow,
    GraphicsContext<gfx_device_gl::CommandBuffer, gfx_device_gl::Device, gfx_device_gl::Factory, gfx_device_gl::Resources>,
)> {
    use glutin::dpi::LogicalSize;
    use glutin::GlRequest;

    let events_loop = glutin::EventsLoop::new();

    let window_builder = glutin::WindowBuilder::new()
        .with_title(title)
        .with_dimensions(LogicalSize::new(width, height));

    let context_builder = glutin::ContextBuilder::new()
        .with_gl(GlRequest::Specific(OpenGl, (3, 2)))
        .with_vsync(true);

    let (window, device, mut factory, color_render_view, depth_stencil_view) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(window_builder, context_builder, &events_loop);

    let command_buffer = factory.create_command_buffer().into();

    Ok((
        events_loop,
        window,
        GraphicsContext {
            color_view: color_render_view,
            depth_view: depth_stencil_view,
            device: device,
            encoder: command_buffer,
            factory: factory,
        },
    ))
}

fn load_diffuse_texture(
    factory: &mut gfx_device_gl::Factory,
    texture_file_path: &str,
) -> (
    ShaderResourceView<gfx_device_gl::Resources, graphics::Vec4>,
    Sampler<gfx_device_gl::Resources>
) {
    match TextureLoader::load_from_file(factory, texture_file_path) {
        Some(texture) => {
            let texture_sampler = factory.create_sampler_linear();
            (texture, texture_sampler)
        },
        None => panic!(format!("Failed loading resource: {}", texture_file_path))
    }
}

fn run() -> Result<()> {
    use glutin::GlContext;

    let (mut events_loop, window, mut gfx_context) = create_gl_window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)?;

    unsafe { window.make_current().unwrap() };

    let pso = gfx_context
        .factory
        .create_pipeline_simple(
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/quad_150.vs.glsl")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/quad_150.fs.glsl")),
            Pipeline::new(),
        ).unwrap();

    let (vertex_buffer, slice) = gfx_context.factory.create_vertex_buffer_with_slice(&SQUARE_VERTS, SQUARE_INDICES);

    let (texture, sampler) = load_diffuse_texture(&mut gfx_context.factory, "resources/uv_grid.jpg");

    let data = Pipeline::Data {
        vbuf: vertex_buffer,
        texture_diffuse: (texture, sampler),
        out_color: gfx_context.color_view,
        // out_depth: depth_stencil_view,
    };

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    _ => {}
                }
            }
        });

        gfx_context.encoder.clear(&data.out_color, CLEAR_COLOR);
        gfx_context.encoder.clear_depth(&gfx_context.depth_view, 1.0);

        gfx_context.encoder.draw(&slice, &pso, &data);

        gfx_context.encoder.flush(&mut gfx_context.device);

        window.swap_buffers().unwrap();
    }

    Ok(())
}

quick_main!(run);
