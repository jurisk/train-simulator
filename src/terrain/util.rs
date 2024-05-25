use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;

// This is unused as we are going for low-poly-style flat normals look instead
#[must_use]
#[allow(unused)]
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

// This is unused as we found `compute_flat_normals`
#[allow(unused)]
fn calculate_triangle_normal(v0: &[f32; 3], v1: &[f32; 3], v2: &[f32; 3]) -> [f32; 3] {
    let edge1 = Vec3::new(v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]);
    let edge2 = Vec3::new(v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]);
    let normal = edge1.cross(edge2).normalize();
    [normal.x, normal.y, normal.z]
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::ptr_arg
)]
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

            normals.push([0.0, 0.0, 0.0]);

            uvs.push([x / x_segments as f32, z / z_segments as f32]);
        }
    }

    // Defining triangles.
    let mut triangles: Vec<u32> = Vec::with_capacity(triangle_count as usize);

    for z_idx in 0 .. z_segments {
        for x_idx in 0 .. x_segments {
            // TODO: Do a similar check as https://github.com/NickToony/gd-retroterrain/blob/master/Terrain.cs#L112 does to improve triangulation

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

    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_triangle_normal_1() {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];

        let expected_normal = [0.0, 0.0, 1.0];
        let calculated_normal = calculate_triangle_normal(&v0, &v1, &v2);

        assert_eq!(calculated_normal, expected_normal);
    }

    #[test]
    fn test_calculate_triangle_normal_2() {
        let v0 = [0.0, 1.0, 1.0];
        let v1 = [1.0, 0.0, 1.0];
        let v2 = [1.0, 1.0, 0.0];

        let expected_normal = [
            1.0 / 3.0_f32.sqrt(),
            1.0 / 3.0_f32.sqrt(),
            1.0 / 3.0_f32.sqrt(),
        ];
        let calculated_normal = calculate_triangle_normal(&v0, &v1, &v2);

        assert_eq!(calculated_normal, expected_normal);
    }
}
