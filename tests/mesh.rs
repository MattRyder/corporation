extern crate libcorporation;
extern crate cgmath;

use libcorporation::mesh::Importer;
use self::cgmath::*;

const MESH_FILE_PATH : &str = "/tests/resources/box.obj";

#[test]
fn should_process_node() {
    const EXPECTED_SCENE_NAME : &str = "box.obj";
    const EXPECTED_MESH_NAME : &str = "TestBoxModel";
    const EXPECTED_MESH_COUNT : usize = 1;
    const EXPECTED_VERTEX_COUNT : usize = 24;

    let mat4_identity = Matrix4::<f32>::identity();

    let file_path = env!("CARGO_MANIFEST_DIR").to_owned() + MESH_FILE_PATH;

    let scene_node_root = Importer::load(&file_path);
    assert!(scene_node_root.is_some());

    // Ensure we have a scene root node:
    let scene_node_root = scene_node_root.unwrap();
    assert_eq!(EXPECTED_SCENE_NAME, scene_node_root.name());
    assert_eq!(&mat4_identity, scene_node_root.transformation());

    // Ensure we have a mesh in there:
    let mesh_node = &scene_node_root.children()[0];
    assert_eq!(EXPECTED_MESH_NAME, mesh_node.name());
    assert_eq!(&mat4_identity, mesh_node.transformation());
    assert_eq!(EXPECTED_MESH_COUNT, mesh_node.meshes().len());

    // Ensure the mesh has the correct attributes:
    let mesh = &mesh_node.meshes()[0];
    assert_eq!(EXPECTED_VERTEX_COUNT, mesh.vertex_iter().len());
}