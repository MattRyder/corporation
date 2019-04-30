pub mod importer;
pub mod scene;
pub mod vertex;

use self::vertex::Vertex;
use std::slice::Iter;

#[derive(Default, Clone)]
pub struct Face {
    pub indices: Vec<u32>,
}

#[derive(Default, Clone)]
pub struct Mesh {
    name: String,
    vertices: Vec<Vertex>,
    faces: Vec<Face>,
}

impl Mesh {
    pub fn new(name: String, vertices: Vec<Vertex>, faces: Vec<Face>) -> Self {
        Mesh { name, vertices, faces }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn indices(&self) -> Vec<u32> {
        self.face_iter().map(|f| f.indices.clone()).flatten().collect()
    }

    pub fn vertex_iter(&self) -> Iter<Vertex> {
        self.vertices.iter()
    }

    pub fn face_iter(&self) -> Iter<Face> {
        self.faces.iter()
    }


}
