use std::fmt::Debug;
use std::ops::Add;

use itertools::Itertools;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use shared_util::bool_ops::BoolResultOps;

use crate::building::BuildError;
use crate::building::building_info::WithTileCoverage;
use crate::building::industry_building_info::IndustryBuildingInfo;
use crate::building::station_info::StationInfo;
use crate::map_level::terrain::Terrain;
use crate::map_level::zoning::{Zoning, ZoningFlattened};
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::transport::track_type::TrackType;
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
    pub const fn from_u8(height: u8) -> Self {
        Self(height)
    }

    #[must_use]
    pub fn fallback() -> Self {
        Self(u8::default())
    }

    #[must_use]
    #[expect(clippy::cast_lossless)]
    pub fn as_f32(self) -> f32 {
        self.0 as f32
    }

    #[must_use]
    pub fn as_u8(self) -> u8 {
        self.0
    }

    #[must_use]
    #[expect(
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

impl Add for Height {
    type Output = Height;

    fn add(self, rhs: Self) -> Self::Output {
        Height(self.0 + rhs.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MapLevel {
    terrain: Terrain,
    water:   Water,
    zoning:  Zoning,
}

impl Serialize for MapLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let flattened: MapLevelFlattened = self.clone().into();
        flattened.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MapLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let flattened = MapLevelFlattened::deserialize(deserializer)?;
        let map_level: MapLevel = flattened.into();
        Ok(map_level)
    }
}

impl From<MapLevel> for MapLevelFlattened {
    fn from(value: MapLevel) -> Self {
        MapLevelFlattened {
            terrain: value.terrain,
            water:   value.water,
            zoning:  value.zoning.into(),
        }
    }
}

impl From<MapLevelFlattened> for MapLevel {
    fn from(value: MapLevelFlattened) -> Self {
        let zoning = value.zoning.into();
        MapLevel::new(value.terrain, value.water, zoning)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct MapLevelFlattened {
    terrain: Terrain,
    water:   Water,
    zoning:  ZoningFlattened,
}

impl MapLevel {
    #[must_use]
    pub fn new(terrain: Terrain, water: Water, zoning: Zoning) -> Self {
        Self {
            terrain,
            water,
            zoning,
        }
    }

    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn load(json: &str) -> Self {
        let map_level = serde_json::from_str::<MapLevel>(json)
            .unwrap_or_else(|err| panic!("Failed to deserialise {json}: {err}"));
        assert_eq!(map_level.is_valid(), Ok(()));
        map_level
    }

    // Could eventually move to some `Validated` instead
    fn is_valid(&self) -> Result<(), String> {
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

    #[expect(clippy::missing_errors_doc)]
    pub fn can_build_track(
        &self,
        tile: TileCoordsXZ,
        track_type: TrackType,
    ) -> Result<(), BuildError> {
        // TODO: We should cache this (have a `Tile` => `TrackTypeSet` grid)
        let vertex_coords = tile.vertex_coords();

        let any_vertex_under_water = vertex_coords
            .into_iter()
            .any(|vertex| self.under_water(vertex));

        any_vertex_under_water.then_err_unit(|| BuildError::InvalidTerrain)?;
        self.terrain.can_build_track(tile, track_type)?;
        self.zoning.can_build_track(tile)?;

        Ok(())
    }

    pub(crate) fn can_build_industry_building(
        &self,
        industry_building_info: &IndustryBuildingInfo,
    ) -> Result<(), BuildError> {
        self.zoning
            .can_build_industry_building(industry_building_info)?;
        self.can_build_for_coverage(&industry_building_info.covers_tiles())?;
        Ok(())
    }

    pub(crate) fn can_build_station(&self, station_info: &StationInfo) -> Result<(), BuildError> {
        self.zoning.can_build_station(station_info)?;
        self.can_build_for_coverage(&station_info.covers_tiles())?;
        Ok(())
    }

    fn can_build_for_coverage(&self, tile_coverage: &TileCoverage) -> Result<(), BuildError> {
        let vertex_coords: Vec<_> = tile_coverage
            .to_set()
            .into_iter()
            .flat_map(TileCoordsXZ::vertex_coords)
            .collect();

        let any_tile_out_of_bounds = tile_coverage
            .to_set()
            .into_iter()
            .any(|tile| !self.tile_in_bounds(tile));

        let any_vertex_under_water = vertex_coords.iter().any(|vertex| self.under_water(*vertex));

        let equal_heights = vertex_coords
            .into_iter()
            .map(|vertex| self.height_at(vertex))
            .all_equal();

        (equal_heights && !any_tile_out_of_bounds && !any_vertex_under_water)
            .then_ok_unit(|| BuildError::InvalidTerrain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_levels_can_be_deserialised() {
        let levels = [
            include_str!("../../../../assets/map_levels/usa_east.json"),
            include_str!("../../../../assets/map_levels/europe.json"),
        ];
        for level_json in levels {
            let level = serde_json::from_str::<MapLevel>(level_json).unwrap_or_else(|err| {
                panic!("Failed to deserialise:\n\n{level_json}\n\nError: {err}")
            });
            assert_eq!(level.is_valid(), Ok(()));
        }
    }
}
