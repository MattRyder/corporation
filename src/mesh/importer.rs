use assimp;
use assimp::math::matrix4::Matrix4x4;
use cgmath::Matrix4;
use mesh::scene::Node;
use mesh::vertex::Vertex;
use mesh::{Face, Mesh};

pub struct Importer {}

impl Importer {
    pub fn load(mesh_file_path: &str) -> Option<Node> {
        use assimp::import::Importer;

        let mut importer = Importer::new();
        importer.triangulate(true);
        importer.generate_normals(|x| x.enable = true);

        if let Ok(scene) = importer.read_file(mesh_file_path) {
            if !scene.is_incomplete() {
                return Some(Self::process_node(&scene, &scene.root_node()));
            }
        }

        None
    }

    fn convert_assimp_matrix(m: Matrix4x4) -> Matrix4<f32> {
        Matrix4::new(
            m.a1, m.a2, m.a3, m.a4, m.b1, m.b2, m.b3, m.b4, m.c1, m.c2, m.c3, m.c4, m.d1, m.d2, m.d3, m.d4,
        )
    }

    fn process_node(scene: &assimp::Scene, node: &assimp::Node) -> Node {
        let scene_node_transform = Self::convert_assimp_matrix(node.transformation());
        let mut scene_node = Node::new(node.name(), scene_node_transform);

        // Load all meshes contained within the assimp Scene Node:
        for mesh_index in node.meshes() {
            if let Some(assimp_mesh) = scene.mesh(*mesh_index as usize) {
                let scene_node_mesh = Self::process_mesh(assimp_mesh);
                scene_node.add_mesh(scene_node_mesh);
            }
        }

        for child_node in node.child_iter() {
            let scene_node_child = Self::process_node(scene, &child_node);
            scene_node.add_child_node(scene_node_child);
        }

        scene_node
    }

    fn process_mesh(assimp_mesh: assimp::Mesh) -> Mesh {
        // Translate assimp datatypes to corporation datatypes:
        let name = String::from_utf8(assimp_mesh.name.data.to_vec()).unwrap();

        let num_vertices = assimp_mesh.num_vertices() as usize;

        let mut vertices = Vec::with_capacity(num_vertices);

        for i in 0..num_vertices {
            let mut position: [f32; 3] = [0.0, 0.0, 0.0];
            let mut tex_coord: [f32; 2] = [0.0, 0.0];

            if assimp_mesh.has_positions() {
                if let Some(ai_vertex) = assimp_mesh.get_vertex(i as u32) {
                    position = [ai_vertex.x, ai_vertex.y, ai_vertex.z];
                }
            }

            // TODO: Implement multiple UV channel lookup
            const UV_CHANNEL_ZERO: usize = 0;

            if let Some(ai_tex_coord) = assimp_mesh.get_texture_coord(UV_CHANNEL_ZERO, i as u32) {
                tex_coord = [ai_tex_coord.x, ai_tex_coord.y];
            }

            vertices.push(Vertex {
                a_Position: position,
                a_TexCoord: tex_coord,
            });
        }

        let faces: Vec<Face> = assimp_mesh
            .face_iter()
            .map(|face| {
                let num_indices = face.num_indices as usize;
                let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

                for i in 0..(num_indices as isize) {
                    indices.push(face[i]);
                }

                Face { indices }
            })
            .collect();

        Mesh { name, vertices, faces }
    }
}
