use maths::vector::{Vec2, Vec3};

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