use std::collections::HashSet;
use std::fmt::{Debug, Formatter};

use log::error;
use serde::{Deserialize, Serialize};

use crate::building::building_info::{
    BuildingDynamicInfo, BuildingInfo, BuildingStaticInfo, WithBuildingDynamicInfo,
    WithBuildingDynamicInfoMut,
};
use crate::building::building_type::BuildingType;
use crate::building::industry_type::IndustryType;
use crate::building::WithRelativeTileCoverage;
use crate::cargo_map::{CargoMap, WithCargo, WithCargoMut};
use crate::game_time::GameTimeDiff;
use crate::resource_type::ResourceType;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::{IndustryBuildingId, PlayerId};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct IndustryBuildingInfo {
    id:            IndustryBuildingId,
    building_type: BuildingType,
    static_info:   BuildingStaticInfo,
    dynamic_info:  BuildingDynamicInfo,
}

impl Debug for IndustryBuildingInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} {:?} {:?}",
            self.id(),
            self.static_info.reference_tile(),
            self.building_type,
            self.dynamic_info
        )
    }
}

impl IndustryBuildingInfo {
    #[must_use]
    pub fn new(
        owner_id: PlayerId,
        id: IndustryBuildingId,
        reference_tile: TileCoordsXZ,
        building_type: BuildingType,
    ) -> Self {
        Self {
            id,
            building_type,
            static_info: BuildingStaticInfo::new(owner_id, reference_tile),
            dynamic_info: BuildingDynamicInfo::new(CargoMap::new()),
        }
    }

    pub fn update_dynamic_info(&mut self, dynamic_info: &BuildingDynamicInfo) {
        *self.dynamic_info_mut() = dynamic_info.clone();
    }

    #[must_use]
    pub fn id(&self) -> IndustryBuildingId {
        self.id
    }

    #[must_use]
    pub fn building_type(&self) -> BuildingType {
        self.building_type
    }

    #[must_use]
    pub fn reference_tile(&self) -> TileCoordsXZ {
        self.static_info.reference_tile()
    }

    pub fn advance_industry_building(&mut self, diff: GameTimeDiff) {
        let seconds = diff.to_seconds();
        match self.building_type() {
            BuildingType::Station(station_id) => {
                error!("Tried to advance industry on a station {station_id:?}");
            },
            BuildingType::Industry(industry_type) => {
                self.advance_industry(seconds, industry_type);
            },
        }
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn industry_transform_inputs(&self) -> HashSet<ResourceType> {
        match self.building_type() {
            BuildingType::Station(_) => HashSet::new(),
            BuildingType::Industry(industry_type) => {
                let mut result = HashSet::new();
                for input in industry_type.transform_per_second().inputs {
                    result.insert(input.resource);
                }
                result
            },
        }
    }

    #[must_use]
    pub fn industry_building_shippable_cargo(&self) -> CargoMap {
        match self.building_type() {
            BuildingType::Station(_) => {
                unreachable!("Should not call this for stations");
            },
            BuildingType::Industry(industry_type) => {
                let transform = industry_type.transform_per_second();
                let mut result = CargoMap::new();
                for output in transform.outputs {
                    let resource = output.resource;
                    // Later: This is now insta-shipping of everything... consider doing this more gradually?
                    let amount = self.dynamic_info().cargo().get(resource);
                    result.add(resource, amount);
                }
                result
            },
        }
    }

    fn advance_industry(&mut self, seconds: f32, industry_type: IndustryType) {
        let transform = industry_type.transform_per_second();
        let utilisation =
            transform.calculate_utilisation_percentage(self.dynamic_info().cargo(), seconds);
        let effective = seconds * utilisation;

        for item in transform.inputs {
            self.dynamic_info_mut()
                .cargo_mut()
                .add(item.resource, -item.amount * effective);
        }
        for item in transform.outputs {
            self.dynamic_info_mut()
                .cargo_mut()
                .add(item.resource, item.amount * effective);
        }
    }
}

impl WithBuildingDynamicInfo for IndustryBuildingInfo {
    fn dynamic_info(&self) -> &BuildingDynamicInfo {
        &self.dynamic_info
    }
}

impl WithBuildingDynamicInfoMut for IndustryBuildingInfo {
    fn dynamic_info_mut(&mut self) -> &mut BuildingDynamicInfo {
        &mut self.dynamic_info
    }
}

impl WithRelativeTileCoverage for IndustryBuildingInfo {
    fn relative_tiles_used(&self) -> TileCoverage {
        self.building_type().relative_tiles_used()
    }
}

impl BuildingInfo for IndustryBuildingInfo {
    fn owner_id(&self) -> PlayerId {
        self.static_info.owner_id()
    }

    fn covers_tiles(&self) -> TileCoverage {
        self.relative_tiles_used().offset_by(self.reference_tile())
    }
}
