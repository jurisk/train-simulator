use std::fmt::Debug;

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::map_level::terrain::Terrain;
use crate::map_level::zoning::Zoning;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::vertex_coords_xz::VertexCoordsXZ;
use crate::water::Water;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TerrainType {
    Sand,
    Grass,
    Rocks,
}

impl TerrainType {
    #[must_use]
    pub fn as_char(&self) -> char {
        match self {
            TerrainType::Sand => 'S',
            TerrainType::Grass => 'G',
            TerrainType::Rocks => 'R',
        }
    }

    #[must_use]
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'S' => Some(Self::Sand),
            'G' => Some(Self::Grass),
            'R' => Some(Self::Rocks),
            _ => None,
        }
    }

    // Have to match the indices for `./assets/textures/land.ktx2`
    #[must_use]
    pub fn as_u32(&self) -> u32 {
        match self {
            TerrainType::Sand => 0,
            TerrainType::Grass => 1,
            TerrainType::Rocks => 2,
        }
    }

    #[must_use]
    pub fn fallback() -> Self {
        Self::Grass
    }
}

impl Serialize for TerrainType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_char().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TerrainType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let c = char::deserialize(deserializer)?;
        Self::from_char(c).ok_or_else(|| Error::custom("Invalid terrain type"))
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
    #[allow(clippy::missing_errors_doc)]
    pub fn is_valid(&self) -> Result<(), String> {
        self.terrain.is_valid()?;
        self.water.is_valid()?;
        Ok(())
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
        let level_json = include_str!("../../../../assets/map_levels/sample.json");
        let level = serde_json::from_str::<MapLevel>(level_json)
            .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));
        assert_eq!(level.is_valid(), Ok(()));
        assert_eq!(level.terrain.vertex_count_x(), 100);
        assert_eq!(level.terrain.vertex_count_z(), 100);
    }
}
