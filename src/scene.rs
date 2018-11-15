use cgmath::*;
use mesh::Mesh;

pub struct Node {
    name: String,
    transformation: Matrix4<f32>,
    meshes: Vec<Mesh>,
    children: Vec<Node>
}

impl Node {
    pub fn new(name: &str, transformation_matrix: Matrix4<f32>) -> Node {
        Node {
            name: name.to_string(),
            transformation: transformation_matrix,
            children: Vec::new(),
            meshes: Vec::new()
        }
    }

    pub fn add_child_node(&mut self, node: Node) {
        self.children.push(node);
    }

    pub fn add_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn transformation(&self) -> &Matrix4<f32> {
        &self.transformation
    }

    pub fn children(&self) -> &Vec<Node> {
        &self.children
    }

    pub fn meshes(&self) -> &Vec<Mesh> {
        &self.meshes
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn should_add_children_to_node() {
        let name = "test";
        let transform = Matrix4::identity();

        let mut root = Node::new(name, transform);
        let child = Node::new(name, transform);
        root.add_child_node(child);

        assert_eq!(1, root.children.len());

    }
}