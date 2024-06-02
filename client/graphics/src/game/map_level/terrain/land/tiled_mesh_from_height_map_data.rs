use bevy::prelude::*;
use bevy::render::mesh::{MeshVertexAttribute, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use shared_util::coords_xz::CoordsXZ;
use shared_util::grid_xz::GridXZ;

#[derive(Copy, Clone)]
struct Vertex {
    position: Vec3,
    uv:       Vec2,
    custom:   u32,
}

#[derive(Copy, Clone)]
struct Triangle {
    vertices: [Vertex; 3],
}

impl Triangle {
    fn new(vertices: [Vertex; 3]) -> Self {
        Self { vertices }
    }
}

// Separate UV maps for each tile
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::ptr_arg,
    clippy::needless_pass_by_value,
    clippy::too_many_arguments
)]
#[must_use]
pub fn tiled_mesh_from_height_map_data<F>(
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
    data: GridXZ<f32>,
    custom_attribute: MeshVertexAttribute,
    custom_f: F,
) -> Mesh
where
    F: Fn(CoordsXZ) -> u32,
{
    let z_segments = data.size_z - 1;
    let x_segments = data.size_x - 1;

    let extent_x = max_x - min_x;
    let extent_z = max_z - min_z;

    let mut triangles: Vec<Triangle> = Vec::with_capacity(x_segments * z_segments);

    for z_idx in 0 .. z_segments {
        for x_idx in 0 .. x_segments {
            const TOP_LEFT_OFFSET: [usize; 2] = [0, 0];
            const TOP_RIGHT_OFFSET: [usize; 2] = [1, 0];
            const BOTTOM_LEFT_OFFSET: [usize; 2] = [0, 1];
            const BOTTOM_RIGHT_OFFSET: [usize; 2] = [1, 1];

            let make_vertex = |offset: [usize; 2]| -> Vertex {
                let x = x_idx + offset[0];
                let z = z_idx + offset[1];
                let coords = CoordsXZ::new(x, z);
                let position = Vec3::new(
                    (x as f32 / x_segments as f32) * extent_x + min_x,
                    data[&coords],
                    (z as f32 / z_segments as f32) * extent_z + min_z,
                );
                let uv = Vec2::new(offset[0] as f32, offset[1] as f32);

                let custom = custom_f(coords);

                Vertex {
                    position,
                    uv,
                    custom,
                }
            };

            let top_left = make_vertex(TOP_LEFT_OFFSET);
            let top_right = make_vertex(TOP_RIGHT_OFFSET);
            let bottom_left = make_vertex(BOTTOM_LEFT_OFFSET);
            let bottom_right = make_vertex(BOTTOM_RIGHT_OFFSET);

            // Similar to https://github.com/NickToony/gd-retroterrain/blob/master/Terrain.cs#L112
            if (top_left.position.y - bottom_right.position.y).abs() < f32::EPSILON {
                triangles.push(Triangle::new([top_left, bottom_left, bottom_right]));
                triangles.push(Triangle::new([top_left, bottom_right, top_right]));
            } else {
                triangles.push(Triangle::new([top_left, bottom_left, top_right]));
                triangles.push(Triangle::new([bottom_left, bottom_right, top_right]));
            }
        }
    }

    convert_to_mesh(triangles, custom_attribute)
}

#[allow(clippy::cast_possible_truncation)]
fn convert_to_mesh(input: Vec<Triangle>, custom_attribute: MeshVertexAttribute) -> Mesh {
    println!("Input: {}", input.len());

    let vertices_count = input.len() * 3;

    let mut triangles: Vec<u32> = Vec::with_capacity(vertices_count);
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertices_count);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertices_count);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertices_count);
    let mut custom: Vec<u32> = Vec::with_capacity(vertices_count);

    for (triangle_idx, triangle) in input.into_iter().enumerate() {
        for (vertex_idx, vertex) in triangle.vertices.into_iter().enumerate() {
            positions.push(vertex.position.into());
            normals.push([0.0, 0.0, 0.0]);
            uvs.push(vertex.uv.into());
            triangles.push((triangle_idx * 3 + vertex_idx) as u32);
            custom.push(vertex.custom);
        }
    }

    println!("Triangles: {:?}", triangles.len());
    println!("Positions: {:?}", positions.len());
    println!("Normals: {:?}", normals.len());
    println!("UVs: {:?}", uvs.len());

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_indices(bevy::render::mesh::Indices::U32(triangles));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh.insert_attribute(custom_attribute, custom);

    mesh

    // Note:    We don't compute normals here, so callers should do it, e.g.:
    //              mesh.duplicate_vertices()
    //              mesh.compute_flat_normals();
}
