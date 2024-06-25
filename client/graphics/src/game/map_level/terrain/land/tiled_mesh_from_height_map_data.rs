use bevy::prelude::*;
use bevy::render::mesh::{MeshVertexAttribute, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::vertex_coords_xz::VertexCoordsXZ;
use shared_util::grid_xz::GridXZ;

#[derive(Copy, Clone, Debug, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub uv:       Vec2,
    pub custom:   u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    vertices: [Vertex; 3],
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Quad {
    pub top_left:     Vertex,
    pub top_right:    Vertex,
    pub bottom_left:  Vertex,
    pub bottom_right: Vertex,
}

impl Quad {
    #[must_use]
    pub fn average_distance_to(&self, point: Vec3) -> f32 {
        let top_left = self.top_left.position;
        let top_right = self.top_right.position;
        let bottom_left = self.bottom_left.position;
        let bottom_right = self.bottom_right.position;

        let distance = |a: Vec3, b: Vec3| (a - b).length();

        (distance(top_left, point)
            + distance(top_right, point)
            + distance(bottom_left, point)
            + distance(bottom_right, point))
            / 4.0
    }
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub quad:      Quad,
    pub triangles: Vec<Triangle>,
}

impl Tile {
    fn empty() -> Self {
        Self {
            quad:      Quad::default(),
            triangles: Vec::new(),
        }
    }
}

#[derive(Clone, Resource)]
pub struct Tiles {
    pub tiles: GridXZ<TileCoordsXZ, Tile>,
}

impl Tiles {
    fn triangles(&self) -> Vec<Triangle> {
        self.tiles
            .coords()
            .flat_map(|coords| self.tiles[coords].triangles.iter().copied())
            .collect()
    }
}

impl Triangle {
    fn new(vertices: [Vertex; 3]) -> Self {
        Self { vertices }
    }
}

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
    data: GridXZ<VertexCoordsXZ, f32>,
    custom_attribute: MeshVertexAttribute,
    custom_f: F,
) -> (Tiles, Mesh)
where
    F: Fn(VertexCoordsXZ) -> u32,
{
    let tiles = tiles_from_heights(min_x, max_x, min_z, max_z, data, custom_f);
    let mesh = convert_to_mesh(&tiles, custom_attribute);
    (tiles, mesh)
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::ptr_arg,
    clippy::needless_pass_by_value,
    clippy::too_many_arguments
)]
#[must_use]
fn tiles_from_heights<F>(
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
    data: GridXZ<VertexCoordsXZ, f32>,
    custom_f: F,
) -> Tiles
where
    F: Fn(VertexCoordsXZ) -> u32,
{
    let z_segments = data.size_z - 1;
    let x_segments = data.size_x - 1;

    let extent_x = max_x - min_x;
    let extent_z = max_z - min_z;

    let mut tiles: GridXZ<TileCoordsXZ, Tile> =
        GridXZ::filled_with(x_segments, z_segments, Tile::empty());

    for z_idx in 0 .. z_segments {
        for x_idx in 0 .. x_segments {
            const TOP_LEFT_OFFSET: [usize; 2] = [0, 0];
            const TOP_RIGHT_OFFSET: [usize; 2] = [1, 0];
            const BOTTOM_LEFT_OFFSET: [usize; 2] = [0, 1];
            const BOTTOM_RIGHT_OFFSET: [usize; 2] = [1, 1];

            let make_vertex = |offset: [usize; 2]| -> Vertex {
                let x = x_idx + offset[0];
                let z = z_idx + offset[1];
                let coords = VertexCoordsXZ::from_usizes(x, z);
                let position = Vec3::new(
                    (x as f32 / x_segments as f32) * extent_x + min_x,
                    data[coords],
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

            let mut triangles = Vec::with_capacity(2);
            let quad = Quad {
                top_left,
                top_right,
                bottom_left,
                bottom_right,
            };

            // Similar to https://github.com/NickToony/gd-retroterrain/blob/master/Terrain.cs#L112
            if (top_left.position.y - bottom_right.position.y).abs() < f32::EPSILON {
                triangles.push(Triangle::new([top_left, bottom_left, bottom_right]));
                triangles.push(Triangle::new([top_left, bottom_right, top_right]));
            } else {
                triangles.push(Triangle::new([top_left, bottom_left, top_right]));
                triangles.push(Triangle::new([bottom_left, bottom_right, top_right]));
            }
            tiles[TileCoordsXZ::from_usizes(x_idx, z_idx)] = Tile { quad, triangles };
        }
    }

    Tiles { tiles }
}

fn calculate_flat_normal(triangle: &Triangle) -> Vec3 {
    let v0 = triangle.vertices[0].position;
    let v1 = triangle.vertices[1].position;
    let v2 = triangle.vertices[2].position;

    let u = v1 - v0;
    let v = v2 - v0;

    u.cross(v).normalize()
}

#[allow(clippy::cast_possible_truncation)]
fn convert_to_mesh(tiles: &Tiles, custom_attribute: MeshVertexAttribute) -> Mesh {
    let input = tiles.triangles();

    trace!("Input: {}", input.len());

    let vertices_count = input.len() * 3;

    let mut triangles: Vec<u32> = Vec::with_capacity(vertices_count);
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertices_count);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertices_count);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertices_count);
    let mut custom: Vec<u32> = Vec::with_capacity(vertices_count);

    for (triangle_idx, triangle) in input.into_iter().enumerate() {
        for (vertex_idx, vertex) in triangle.vertices.into_iter().enumerate() {
            positions.push(vertex.position.into());
            normals.push(calculate_flat_normal(&triangle).into());
            uvs.push(vertex.uv.into());
            triangles.push((triangle_idx * 3 + vertex_idx) as u32);
            custom.push(vertex.custom);
        }
    }

    trace!("Triangles: {:?}", triangles.len());
    trace!("Positions: {:?}", positions.len());
    trace!("Normals: {:?}", normals.len());
    trace!("UVs: {:?}", uvs.len());

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_flat_normal() {
        let triangle = Triangle {
            vertices: [
                Vertex {
                    position: Vec3::new(0.0, 0.0, 0.0),
                    uv:       Vec2::new(0.0, 0.0),
                    custom:   0,
                },
                Vertex {
                    position: Vec3::new(0.0, 1.0, 0.0),
                    uv:       Vec2::new(0.0, 1.0),
                    custom:   0,
                },
                Vertex {
                    position: Vec3::new(1.0, 0.0, 0.0),
                    uv:       Vec2::new(1.0, 0.0),
                    custom:   0,
                },
            ],
        };

        let normal = calculate_flat_normal(&triangle);
        assert_eq!(normal, Vec3::new(0.0, 0.0, -1.0));
    }
}
