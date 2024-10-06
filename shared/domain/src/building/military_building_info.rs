use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::building::WithRelativeTileCoverage;
use crate::building::building_info::{BuildingInfo, WithCostToBuild, WithOwner, WithTileCoverage};
use crate::building::industry_type::IndustryType;
use crate::building::military_building_type::MilitaryBuildingType;
use crate::cargo_map::CargoMap;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::{MilitaryBuildingId, PlayerId};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct MilitaryBuildingInfo {
    id:                     MilitaryBuildingId,
    owner_id:               PlayerId,
    military_building_type: MilitaryBuildingType,
    reference_tile:         TileCoordsXZ,
}

impl Debug for MilitaryBuildingInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} {:?}",
            self.id(),
            self.reference_tile,
            self.military_building_type,
        )
    }
}

impl MilitaryBuildingInfo {
    #[must_use]
    pub fn new(
        id: MilitaryBuildingId,
        owner_id: PlayerId,
        military_building_type: MilitaryBuildingType,
        reference_tile: TileCoordsXZ,
    ) -> Self {
        Self {
            id,
            owner_id,
            military_building_type,
            reference_tile,
        }
    }

    #[must_use]
    pub fn id(&self) -> MilitaryBuildingId {
        self.id
    }

    #[must_use]
    pub fn military_building_type(&self) -> MilitaryBuildingType {
        self.military_building_type
    }

    #[must_use]
    pub fn reference_tile(&self) -> TileCoordsXZ {
        self.reference_tile
    }
}

impl WithRelativeTileCoverage for MilitaryBuildingInfo {
    fn relative_tiles_used(&self) -> TileCoverage {
        self.military_building_type.relative_tiles_used()
    }
}

impl WithTileCoverage for MilitaryBuildingInfo {
    fn covers_tiles(&self) -> TileCoverage {
        self.relative_tiles_used().offset_by(self.reference_tile())
    }
}

impl WithCostToBuild for MilitaryBuildingInfo {
    fn cost_to_build(&self) -> (IndustryType, CargoMap) {
        self.military_building_type.cost_to_build()
    }
}

impl WithOwner for MilitaryBuildingInfo {
    fn owner_id(&self) -> PlayerId {
        self.owner_id
    }
}

impl BuildingInfo for MilitaryBuildingInfo {}
