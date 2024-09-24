#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use shared_util::bool_ops::BoolResultOps;
use shared_util::grid_xz::GridXZ;

use crate::ZoningId;
use crate::building::building_info::WithTileCoverage;
use crate::building::industry_building_info::IndustryBuildingInfo;
use crate::building::station_info::StationInfo;
use crate::building::{BuildError, WithRelativeTileCoverage};
use crate::map_level::zoning::ZoningType::Source;
use crate::resource_type::ResourceType;
use crate::resource_type::ResourceType::{
    Clay, Coal, FarmProducts, Iron, Limestone, Nitrates, Oil, SandAndGravel, Sulfur, Wood,
};
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;

#[derive(Serialize, Deserialize, Hash, Copy, Clone, Eq, PartialEq)]
pub enum ZoningType {
    Source(ResourceType),
    Industrial,
}

impl Debug for ZoningType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Source(resource_type) => write!(f, "{resource_type:?}"),
            ZoningType::Industrial => write!(f, "Industrial"),
        }
    }
}

impl ZoningType {
    #[must_use]
    pub const fn all() -> [ZoningType; 11] {
        [
            Source(Clay),
            Source(Coal),
            Source(FarmProducts), // Farm-land?
            Source(Iron),
            Source(Limestone),
            Source(Nitrates),
            Source(Oil),
            Source(SandAndGravel),
            Source(Sulfur),
            Source(Wood),
            ZoningType::Industrial,
        ]
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ZoningInfo {
    id:             ZoningId,
    zoning_type:    ZoningType,
    reference_tile: TileCoordsXZ,
}

impl ZoningInfo {
    #[must_use]
    pub fn new(id: ZoningId, zoning_type: ZoningType, reference_tile: TileCoordsXZ) -> Self {
        Self {
            id,
            zoning_type,
            reference_tile,
        }
    }

    #[must_use]
    pub fn id(&self) -> ZoningId {
        self.id
    }

    #[must_use]
    pub fn zoning_type(&self) -> ZoningType {
        self.zoning_type
    }

    #[must_use]
    pub fn reference_tile(&self) -> TileCoordsXZ {
        self.reference_tile
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ZoningFlattened {
    size_x: usize,
    size_z: usize,
    infos:  Vec<ZoningInfo>,
}

impl From<Zoning> for ZoningFlattened {
    fn from(value: Zoning) -> Self {
        let mut infos = Vec::new();
        for info in value.all_zonings() {
            infos.push(info.clone());
        }
        Self {
            size_x: value.grid.size_x,
            size_z: value.grid.size_z,
            infos,
        }
    }
}

impl From<ZoningFlattened> for Zoning {
    fn from(value: ZoningFlattened) -> Self {
        let mut result = Zoning::new(value.size_x, value.size_z);
        for info in value.infos {
            result.add_zoning(info);
        }
        result
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Zoning {
    infos: HashMap<ZoningId, ZoningInfo>,
    grid:  GridXZ<TileCoordsXZ, Option<ZoningId>>,
}

impl Zoning {
    #[must_use]
    pub fn new(size_x: usize, size_z: usize) -> Self {
        Self {
            infos: HashMap::new(),
            grid:  GridXZ::filled_with(size_x, size_z, None),
        }
    }

    pub fn add_zoning(&mut self, zoning_info: ZoningInfo) {
        let coverage = zoning_info.covers_tiles();
        let id = zoning_info.id;
        self.infos.insert(id, zoning_info);
        for tile in coverage.to_set() {
            self.grid[tile] = Some(id);
        }
    }

    #[must_use]
    pub fn all_zonings(&self) -> Vec<&ZoningInfo> {
        self.infos.values().collect()
    }

    #[must_use]
    pub fn zoning_at_reference_tile(&self, tile: TileCoordsXZ) -> Option<&ZoningInfo> {
        let found = self.grid.get(tile)?;
        found.and_then(|id| self.infos.get(&id))
    }

    #[must_use]
    pub fn free_at_tile(&self, tile: TileCoordsXZ) -> bool {
        match self.grid.get(tile) {
            None => false,
            Some(found) => found.is_none(),
        }
    }

    #[expect(clippy::missing_errors_doc)]
    pub fn can_build_industry_building(
        &self,
        industry_building_info: &IndustryBuildingInfo,
    ) -> Result<(), BuildError> {
        let required = industry_building_info.required_zoning();
        if required.is_some() {
            (self
                .zoning_at_reference_tile(industry_building_info.reference_tile())
                .map(|zoning| zoning.zoning_type)
                == required)
                .then_ok_unit(|| BuildError::InvalidZoning)
        } else {
            (industry_building_info
                .covers_tiles()
                .to_set()
                .iter()
                .all(|tile| self.free_at_tile(*tile)))
            .then_ok_unit(|| BuildError::InvalidOverlap)
        }
    }

    #[inline]
    #[expect(clippy::missing_errors_doc)]
    pub fn can_build_track(&self, tile: TileCoordsXZ) -> Result<(), BuildError> {
        self.free_at_tile(tile)
            .then_ok_unit(|| BuildError::InvalidOverlap)
    }

    #[expect(clippy::missing_errors_doc)]
    pub fn can_build_station(&self, station_info: &StationInfo) -> Result<(), BuildError> {
        station_info
            .covers_tiles()
            .to_set()
            .iter()
            .all(|tile| self.free_at_tile(*tile))
            .then_ok_unit(|| BuildError::InvalidOverlap)
    }
}

impl WithRelativeTileCoverage for ZoningType {
    fn relative_tiles_used(&self) -> TileCoverage {
        TileCoverage::rectangular_odd(1)
    }
}

impl WithTileCoverage for ZoningInfo {
    fn covers_tiles(&self) -> TileCoverage {
        self.zoning_type
            .relative_tiles_used()
            .offset_by(self.reference_tile)
    }
}
