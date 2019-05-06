use mesh::scene::Node;
use camera::Camera;
use graphics::gfx_hal::image::Image;

pub mod gfx_hal;

pub struct UniformInitializer<T: Copy> {
    pub data: Vec<T>,
}

pub struct RenderStateInitializer<T: Copy> {
    pub camera: Camera<f32>,
    pub textures: Vec<(u32, Image)>,
    pub uniforms: Vec<(u32, UniformInitializer<T>)>,
    pub mesh_node: Node,
}