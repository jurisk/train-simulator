use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::terrain::Terrain;
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
pub struct Height(pub u8);

impl Height {
    #[must_use]
    pub fn fallback() -> Self {
        Self(u8::default())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MapLevel {
    terrain: Terrain,
    water:   Water,
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
        let level_json = include_str!("../../../assets/map_levels/default.json");
        let level = serde_json::from_str::<MapLevel>(level_json)
            .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));
        assert!(level.is_valid());
        assert_eq!(level.terrain.vertex_count_x(), 100);
        assert_eq!(level.terrain.vertex_count_z(), 100);
    }
}
