use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::map_level::terrain::Terrain;
use crate::map_level::zoning::Zoning;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::vertex_coords_xz::VertexCoordsXZ;
use crate::water::Water;

#[repr(u32)]
pub enum TerrainType {
    Sand  = 0,
    Grass = 1,
    Rocks = 2,
}

impl TerrainType {
    #[must_use]
    pub fn default_from_height(height: Height) -> Self {
        if height.0 <= 9 {
            TerrainType::Sand
        } else if height.0 <= 15 {
            TerrainType::Grass
        } else {
            TerrainType::Rocks
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Height(u8);

impl Height {
    #[must_use]
    pub fn from_u8(height: u8) -> Self {
        Self(height)
    }

    #[must_use]
    pub fn fallback() -> Self {
        Self(u8::default())
    }

    #[must_use]
    #[allow(clippy::cast_lossless)]
    pub fn as_f32(self) -> f32 {
        self.0 as f32
    }

    #[must_use]
    pub fn as_u8(self) -> u8 {
        self.0
    }

    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn average_rounded(heights: &[Self]) -> Self {
        let sum: f32 = heights.iter().map(|&height| height.as_f32()).sum();
        let average = sum / heights.len() as f32;
        let rounded = average.round() as u8;
        Self(rounded)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MapLevel {
    terrain: Terrain,
    water:   Water,
    zoning:  Zoning,
}

impl MapLevel {
    // Could eventually move to some `Validated` instead
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.terrain.is_valid() && self.water.is_valid()
    }

    #[must_use]
    pub fn terrain(&self) -> &Terrain {
        &self.terrain
    }

    #[must_use]
    pub fn water(&self) -> &Water {
        &self.water
    }

    #[must_use]
    pub fn zoning(&self) -> &Zoning {
        &self.zoning
    }

    #[must_use]
    pub fn height_at(&self, vertex_coords_xz: VertexCoordsXZ) -> Height {
        self.terrain.height_at(vertex_coords_xz)
    }

    #[must_use]
    pub fn under_water(&self, vertex_coords_xz: VertexCoordsXZ) -> bool {
        self.water.under_water(self.height_at(vertex_coords_xz))
    }

    #[must_use]
    pub fn tile_in_bounds(&self, tile: TileCoordsXZ) -> bool {
        self.terrain.tile_in_bounds(tile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_terrain_can_be_deserialised() {
        let level_json = include_str!("../../../../assets/map_levels/default.json");
        let level = serde_json::from_str::<MapLevel>(level_json)
            .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));
        assert!(level.is_valid());
        assert_eq!(level.terrain.vertex_count_x(), 100);
        assert_eq!(level.terrain.vertex_count_z(), 100);
    }
}
