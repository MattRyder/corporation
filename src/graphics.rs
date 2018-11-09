use gfx;
use gfx::handle::ShaderResourceView;
use gfx::texture::{AaMode, Kind, Mipmap};
use image;

pub type Vec2 = [f32; 2];
pub type Vec3 = [f32; 3];
pub type Vec4 = [f32; 4];

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;
pub type RenderTargetView<R> = gfx::handle::RenderTargetView<R, ColorFormat>;
pub type DepthTargetView<R> = gfx::handle::DepthStencilView<R, DepthFormat>;

pub struct GraphicsContext<C: gfx::CommandBuffer<R>, D: gfx::Device, F: gfx::traits::FactoryExt<R> + Clone, R: gfx::Resources> {
    pub color_view: RenderTargetView<R>,
    pub depth_view: DepthTargetView<R>,
    pub device: D,
    pub encoder: gfx::Encoder<R, C>,
    #[allow(dead_code)]
    pub factory: F,
}

gfx_defines! {

    #[derive(Default)]
    vertex Vertex {
        position: Vec3 = "a_Position",
        //normal: Vec3 = "a_Normal",
        tex_coord: Vec2 = "a_TexCoord",
    }

    constant Locals {
        transform: [Vec4; 4] = "u_Transform",
    }

    pipeline Pipeline {
        vbuf: gfx::VertexBuffer<Vertex> = (),

        // Added for compatibility with devices that don't support const buffers
        // transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",

        // Like Uniform Buffer storage in opengl
        // locals: gfx::ConstantBuffer<Locals> = "Locals",

        texture_diffuse: gfx::TextureSampler<Vec4> = "t_Diffuse",

        out_color: gfx::RenderTarget<ColorFormat> = "Target0",

        //out_depth: gfx::RenderTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

pub struct TextureLoader {}

impl TextureLoader {

    ///
    /// Loads a texture from file
    ///
    pub fn load_from_file<F, R>(factory: &mut F, image_file_path: &str) -> Option<ShaderResourceView<R, Vec4>>
    where
        F: gfx::Factory<R>,
        R: gfx::Resources,
    {
        match image::open(image_file_path) {
            Ok(image) => {
                let image = image.flipv().to_rgba();
                let (width, height) = image.dimensions();
                let kind = Kind::D2(width as u16, height as u16, AaMode::Single);

                match factory.create_texture_immutable_u8::<ColorFormat>(kind, Mipmap::Provided, &[&image.into_raw()[..]]) {
                    Ok((_, view)) => Some(view),
                    Err(create_texture_error) => panic!(create_texture_error) 
                }
            }
            Err(image_error) => panic!(image_error)
        }
    }
}
