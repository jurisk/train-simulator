use std::fmt::{Debug, Formatter};

use bevy_math::Vec3;
use serde::{Deserialize, Serialize};
use shared_util::coords_xz::CoordsXZ;
use shared_util::direction_xz::DirectionXZ;
use shared_util::grid_xz::GridXZ;

use crate::map_level::Height;
use crate::transport::tile_track::TileTrack;
use crate::vertex_coords_xz::VertexCoordsXZ;
use crate::TileCoordsXZ;

// TODO: Make fields private
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Terrain {
    y_coef:         f32,
    vertex_heights: GridXZ<VertexCoordsXZ, Height>,
}

impl Debug for Terrain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Terrain")
            .field("size_x", &self.vertex_heights.size_x)
            .field("size_z", &self.vertex_heights.size_z)
            .field("y_coef", &self.y_coef)
            .finish()
    }
}

pub const DEFAULT_Y_COEF: f32 = 0.5;
impl Terrain {
    #[must_use]
    pub fn new(vertex_heights: Vec<Vec<u8>>) -> Self {
        Self {
            y_coef:         DEFAULT_Y_COEF,
            vertex_heights: GridXZ::new(vertex_heights).map(|&height| Height::from_u8(height)),
        }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.vertex_heights.is_valid()
    }

    #[must_use]
    pub fn vertex_count_x(&self) -> usize {
        self.vertex_heights.size_x
    }

    #[must_use]
    pub fn vertex_count_z(&self) -> usize {
        self.vertex_heights.size_z
    }

    #[must_use]
    pub fn tile_count_x(&self) -> usize {
        self.vertex_count_x() - 1
    }

    #[must_use]
    pub fn tile_count_z(&self) -> usize {
        self.vertex_count_z() - 1
    }

    #[must_use]
    pub fn edge_center_coordinate(&self, direction: DirectionXZ, tile: TileCoordsXZ) -> Vec3 {
        let (a, b) = self.vertex_coordinates_clockwise(direction, tile);
        (a + b) / 2.0
    }

    #[must_use]
    pub fn vertex_coordinates_clockwise(
        &self,
        direction: DirectionXZ,
        tile: TileCoordsXZ,
    ) -> (Vec3, Vec3) {
        let (a, b) = tile.vertex_coords_clockwise(direction);
        (self.logical_to_world(a), self.logical_to_world(b))
    }

    #[must_use]
    pub fn height_at(&self, vertex_coords_xz: VertexCoordsXZ) -> Height {
        match self.vertex_heights.get(vertex_coords_xz) {
            Some(&height) => height,
            None => Height::fallback(),
        }
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
    pub fn logical_to_world(&self, vertex_coords_xz: VertexCoordsXZ) -> Vec3 {
        let height = self.vertex_heights[vertex_coords_xz].as_f32();
        let coords_xz: CoordsXZ = vertex_coords_xz.into();
        let y = height * self.y_coef;
        let x = (coords_xz.x as f32) - (self.tile_count_x() as f32) / 2.0;
        let z = (coords_xz.z as f32) - (self.tile_count_z() as f32) / 2.0;
        Vec3::new(x, y, z)
    }

    #[must_use]
    pub fn entry_and_exit(&self, tile_track: TileTrack) -> (Vec3, Vec3) {
        let tile = tile_track.tile_coords_xz;
        let track_type = tile_track.track_type;
        let exit_direction = tile_track.pointing_in;
        let entry_direction = track_type.other_end(exit_direction);
        let entry = self.edge_center_coordinate(entry_direction, tile);
        let exit = self.edge_center_coordinate(exit_direction, tile);
        (entry, exit)
    }

    #[must_use]
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub fn tile_in_bounds(&self, tile: TileCoordsXZ) -> bool {
        let coords: CoordsXZ = tile.into();
        let valid_x = coords.x >= 0 && coords.x < self.tile_count_x() as i32;
        let valid_z = coords.z >= 0 && coords.z < self.tile_count_z() as i32;
        valid_x && valid_z
    }

    #[must_use]
    pub fn y_coef(&self) -> f32 {
        self.y_coef
    }
}
