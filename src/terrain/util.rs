use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;

#[must_use]
#[allow(clippy::ptr_arg)]
fn normal_from_height_map(
    x: usize,
    z: usize,
    data: &Vec<Vec<f32>>,
    x_segments: usize,
    z_segments: usize,
) -> Vec3 {
    // Calculate slopes in the x and z directions
    let dx = if x > 0 && x < x_segments - 1 {
        data[z][x + 1] - data[z][x - 1]
    } else {
        0.0
    };

    let dz = if z > 0 && z < z_segments - 1 {
        data[z + 1][x] - data[z - 1][x]
    } else {
        0.0
    };

    // Calculate normal with unit length
    Vec3::new(-dx, 1.0, -dz).normalize()
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
#[must_use]
pub fn mesh_from_height_map_data(
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
    data: &Vec<Vec<f32>>,
) -> Mesh {
    let z_segments = data.len() as u32 - 1;
    let x_segments = data[0].len() as u32 - 1;

    let vertices_count: u32 = (x_segments + 1) * (z_segments + 1);
    let triangle_count: u32 = x_segments * z_segments * 2 * 3;

    // Defining vertices.
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertices_count as usize);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertices_count as usize);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertices_count as usize);

    let extent_x = max_x - min_x;
    let extent_z = max_z - min_z;

    for z_idx in 0 ..= z_segments {
        for x_idx in 0 ..= x_segments {
            let (x, z) = (x_idx as f32, z_idx as f32);

            let pos = [
                (x / x_segments as f32) * extent_x + min_x,
                data[z_idx as usize][x_idx as usize],
                (z / z_segments as f32) * extent_z + min_z,
            ];

            debug_assert!(pos[0] >= min_x);
            debug_assert!(pos[0] <= max_x);
            debug_assert!(pos[2] >= min_z);
            debug_assert!(pos[2] <= max_z);
            positions.push(pos);

            let normal = normal_from_height_map(
                x_idx as usize,
                z_idx as usize,
                data,
                x_segments as usize,
                z_segments as usize,
            );
            normals.push(normal.into());

            uvs.push([x / x_segments as f32, z / z_segments as f32]);
        }
    }

    // Defining triangles.
    let mut triangles: Vec<u32> = Vec::with_capacity(triangle_count as usize);

    for z_idx in 0 .. z_segments {
        for x_idx in 0 .. x_segments {
            // First triangle
            triangles.push((z_idx * (x_segments + 1)) + x_idx);
            triangles.push(((z_idx + 1) * (x_segments + 1)) + x_idx);
            triangles.push(((z_idx + 1) * (x_segments + 1)) + x_idx + 1);
            // Second triangle
            triangles.push((z_idx * (x_segments + 1)) + x_idx);
            triangles.push(((z_idx + 1) * (x_segments + 1)) + x_idx + 1);
            triangles.push((z_idx * (x_segments + 1)) + x_idx + 1);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_indices(bevy::render::mesh::Indices::U32(triangles));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh
}