use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use shared_util::grid_xz::GridXZ;

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

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
pub struct Height(pub u8);

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Terrain {
    pub vertex_heights: GridXZ<Height>,
}

impl Debug for Terrain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Terrain")
            .field("size_x", &self.vertex_heights.size_x)
            .field("size_z", &self.vertex_heights.size_z)
            .finish()
    }
}

impl Terrain {
    #[must_use]
    pub fn new(vertex_heights: Vec<Vec<u8>>) -> Self {
        Self {
            vertex_heights: GridXZ::new(vertex_heights).map(|&height| Height(height)),
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
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Water {
    pub between: (Height, Height),
}

impl Water {
    fn is_valid(&self) -> bool {
        let (below, above) = &self.between;
        below.0 + 1 == above.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct MapLevel {
    pub terrain: Terrain,
    pub water:   Water,
}

impl MapLevel {
    // Could eventually move to some `Validated` instead
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.terrain.is_valid() && self.water.is_valid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_terrain_can_be_deserialised() {
        let level_json = include_str!("../../../game/logic/assets/map_levels/default.json");
        let level = serde_json::from_str::<MapLevel>(level_json)
            .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));
        assert!(level.is_valid());
        assert_eq!(level.terrain.vertex_count_x(), 100);
        assert_eq!(level.terrain.vertex_count_z(), 100);
    }
}
