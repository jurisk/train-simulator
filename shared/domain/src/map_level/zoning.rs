#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};

use crate::building::building_info::WithTileCoverage;
use crate::building::industry_building_info::IndustryBuildingInfo;
use crate::building::station_info::StationInfo;
use crate::building::WithRelativeTileCoverage;
use crate::map_level::zoning::ZoningType::Source;
use crate::resource_type::ResourceType;
use crate::resource_type::ResourceType::{
    Clay, Coal, FarmProducts, Iron, Limestone, Nitrates, Oil, SandAndGravel, Sulfur, Wood,
};
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::ZoningId;

#[derive(Serialize, Deserialize, Hash, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ZoningType {
    Source(ResourceType),
    Industrial,
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

// TODO HIGH: Optimise
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Zoning(Vec<ZoningInfo>);

impl Zoning {
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[must_use]
    pub fn all_zonings(&self) -> &Vec<ZoningInfo> {
        &self.0
    }

    #[must_use]
    pub fn zoning_at_reference_tile(&self, tile: TileCoordsXZ) -> Option<&ZoningInfo> {
        self.all_zonings()
            .iter()
            .find(|zoning_info| zoning_info.reference_tile == tile)
    }

    #[must_use]
    pub fn zoning_at_tile(&self, tile: TileCoordsXZ) -> Option<&ZoningInfo> {
        self.all_zonings()
            .iter()
            .find(|zoning_info| zoning_info.covers_tiles().contains(tile))
    }

    #[must_use]
    pub fn can_build_industry_building(
        &self,
        industry_building_info: &IndustryBuildingInfo,
    ) -> bool {
        let zoning_info = self.zoning_at_reference_tile(industry_building_info.reference_tile());
        match zoning_info {
            Some(zoning_info) => {
                industry_building_info.required_zoning() == zoning_info.zoning_type
            },
            None => false,
        }
    }

    // TODO HIGH: Optimise
    #[must_use]
    pub fn can_build_track(&self, tile: TileCoordsXZ) -> bool {
        self.zoning_at_tile(tile).is_none()
    }

    #[must_use]
    pub fn can_build_station(&self, station_info: &StationInfo) -> bool {
        station_info
            .covers_tiles()
            .to_set()
            .iter()
            .all(|tile| self.zoning_at_tile(*tile).is_none())
    }
}

impl WithRelativeTileCoverage for ZoningType {
    fn relative_tiles_used(&self) -> TileCoverage {
        TileCoverage::Rectangular {
            north_west_inclusive: TileCoordsXZ::new(-1, -1),
            south_east_inclusive: TileCoordsXZ::new(1, 1),
        }
    }
}

impl WithTileCoverage for ZoningInfo {
    fn covers_tiles(&self) -> TileCoverage {
        self.zoning_type
            .relative_tiles_used()
            .offset_by(self.reference_tile)
    }
}
