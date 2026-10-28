use std::path::PathBuf;

use tracecraft_lib::commands::pcb::get_3d_model_data;

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("tracecraft-model-tests");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn unique(name: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    temp_dir().join(format!("{}-{}", stamp, name))
}

fn positions_bytes(positions: &[[f32; 3]]) -> Vec<u8> {
    let mut out = Vec::new();
    for vertex in positions {
        for component in vertex {
            out.extend_from_slice(&component.to_le_bytes());
        }
    }
    out
}

#[test]
fn loads_ascii_stl() {
    let path = unique("tri.stl");
    std::fs::write(
        &path,
        "solid tri\n facet normal 0 0 1\n  outer loop\n   vertex 0 0 0\n   vertex 1 0 0\n   vertex 0 1 0\n  endloop\n endfacet\nendsolid tri\n",
    )
    .unwrap();
    let model = get_3d_model_data(path.to_string_lossy().to_string()).unwrap();
    assert_eq!(model.vertices.len(), 9);
    assert_eq!(model.normals.len(), 9);
    assert!((model.vertices[3] - 1.0).abs() < 1e-6);
}

#[test]
fn loads_gltf_with_external_buffer() {
    let path = unique("tri.gltf");
    let bin_path = path.with_extension("bin");
    let positions = [[0.0f32, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
    std::fs::write(&bin_path, positions_bytes(&positions)).unwrap();

    let gltf = r#"{
        "asset": {"version": "2.0"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0}}]}],
        "buffers": [{"uri": "BINNAME", "byteLength": 36}],
        "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": 36}],
        "accessors": [{"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0,0,0], "max": [2,3,0]}]
    }"#
    .replace("BINNAME", bin_path.file_name().unwrap().to_str().unwrap());

    std::fs::write(&path, gltf).unwrap();
    let model = get_3d_model_data(path.to_string_lossy().to_string()).unwrap();
    assert_eq!(model.vertices.len(), 9);
    assert_eq!(model.normals.len(), 9);
    // Second vertex is at x=2.
    assert!((model.vertices[3] - 2.0).abs() < 1e-6);
}

#[test]
fn loads_glb_with_indices() {
    let path = unique("quad.glb");

    let positions = [
        [0.0f32, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];
    let mut bin = positions_bytes(&positions);
    for index in [0u16, 1, 2, 0, 2, 3] {
        bin.extend_from_slice(&index.to_le_bytes());
    }

    let json = r#"{
        "asset": {"version": "2.0"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0}, "indices": 1}]}],
        "buffers": [{"byteLength": 60}],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": 48},
            {"buffer": 0, "byteOffset": 48, "byteLength": 12}
        ],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": 4, "type": "VEC3", "min": [0,0,0], "max": [1,1,0]},
            {"bufferView": 1, "componentType": 5123, "count": 6, "type": "SCALAR"}
        ]
    }"#;

    let mut json_bytes = json.as_bytes().to_vec();
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }
    while bin.len() % 4 != 0 {
        bin.push(0);
    }
    let total = 12 + 8 + json_bytes.len() + 8 + bin.len();
    let mut glb = Vec::new();
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total as u32).to_le_bytes());
    glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"JSON");
    glb.extend_from_slice(&json_bytes);
    glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    glb.extend_from_slice(b"BIN\0");
    glb.extend_from_slice(&bin);

    std::fs::write(&path, glb).unwrap();
    let model = get_3d_model_data(path.to_string_lossy().to_string()).unwrap();
    // Two triangles, expanded from the index buffer.
    assert_eq!(model.vertices.len(), 18);
    assert_eq!(model.normals.len(), 18);
    // Every generated normal on this flat quad should point along +Z.
    for chunk in model.normals.chunks_exact(3) {
        assert!(chunk[2].abs() > 0.9, "normal {:?}", chunk);
    }
}

#[test]
fn rejects_unknown_extension() {
    let path = unique("thing.step");
    std::fs::write(&path, b"whatever").unwrap();
    let error = get_3d_model_data(path.to_string_lossy().to_string()).unwrap_err();
    assert!(error.contains("Unsupported 3D model format"));
}
