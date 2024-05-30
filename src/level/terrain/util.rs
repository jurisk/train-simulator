use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::ptr_arg,
    clippy::needless_pass_by_value
)]
#[must_use]
pub fn mesh_from_height_map_data(
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
    y_coef: f32,
    data: Vec<Vec<f32>>,
) -> Mesh {
    let z_segments = data.len() as u32 - 1;
    let x_segments = data[0].len() as u32 - 1;

    let vertices_count: u32 = (x_segments + 1) * (z_segments + 1);
    let triangle_count: u32 = x_segments * z_segments * 2 * 3;

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
                data[z_idx as usize][x_idx as usize] * y_coef,
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

    let mut triangles: Vec<u32> = Vec::with_capacity(triangle_count as usize);

    for z_idx in 0 .. z_segments {
        for x_idx in 0 .. x_segments {
            const Y_IDX: usize = 1;

            let top_left = z_idx * (x_segments + 1) + x_idx;
            let bottom_left = (z_idx + 1) * (x_segments + 1) + x_idx;
            let bottom_right = (z_idx + 1) * (x_segments + 1) + x_idx + 1;
            let top_right = z_idx * (x_segments + 1) + x_idx + 1;

            // Similar to https://github.com/NickToony/gd-retroterrain/blob/master/Terrain.cs#L112
            if (positions[top_left as usize][Y_IDX] - positions[bottom_right as usize][Y_IDX]).abs()
                < f32::EPSILON
            {
                triangles.extend_from_slice(&[top_left, bottom_left, bottom_right]);
                triangles.extend_from_slice(&[top_left, bottom_right, top_right]);
            } else {
                triangles.extend_from_slice(&[top_left, bottom_left, top_right]);
                triangles.extend_from_slice(&[bottom_left, bottom_right, top_right]);
            }
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
