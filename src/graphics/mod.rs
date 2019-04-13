pub mod shader;
pub mod texture;

pub mod adapter;
pub mod backend;
pub mod buffer;
pub mod descriptor;
pub mod device;
pub mod framebuffer;
pub mod image;
pub mod pipeline;
pub mod renderer;
pub mod swapchain;
pub mod uniform;
pub mod window;

pub type Vec2 = [f32; 2];
pub type Vec3 = [f32; 3];
pub type Vec4 = [f32; 4];

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct Vertex {
    pub a_Position: Vec3,
    pub a_TexCoord: Vec2,
}

impl Vertex {
    pub fn set_position(&mut self, position: Vec3) {
        self.a_Position = position;
    }

    pub fn set_tex_coord(&mut self, tex_coord: Vec2) {
        self.a_TexCoord = tex_coord;
    }
}
