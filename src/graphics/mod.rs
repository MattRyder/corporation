pub mod texture;

use gfx;

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

    constant Camera {
        projection: [Vec4; 4] = "u_Projection",
        view: [Vec4; 4] = "u_View",
    }

    pipeline Pipeline {
        vbuf: gfx::VertexBuffer<Vertex> = (),

        // Added for compatibility with devices that don't support const buffers
        // projection_matrix: gfx::Global<[Vec4; 4]> = "u_Projection",
        // transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",

        // Like Uniform Buffer storage in opengl
        camera: gfx::ConstantBuffer<Camera> = "u_Camera",

        texture_diffuse: gfx::TextureSampler<Vec4> = "t_Diffuse",

        out_color: gfx::RenderTarget<ColorFormat> = "Target0",

        out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl Vertex {

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn set_tex_coord(&mut self, tex_coord: Vec2) {
        self.tex_coord = tex_coord;
    }

}