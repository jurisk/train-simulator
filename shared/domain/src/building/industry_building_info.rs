use std::collections::HashSet;
use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::building::building_info::{
    BuildingDynamicInfo, BuildingInfo, BuildingStaticInfo, WithBuildingDynamicInfo,
    WithBuildingDynamicInfoMut, WithOwner, WithTileCoverage,
};
use crate::building::industry_type::IndustryType;
use crate::building::station_type::StationType;
use crate::building::WithRelativeTileCoverage;
use crate::cargo_map::{CargoMap, WithCargo, WithCargoMut};
use crate::game_time::GameTimeDiff;
use crate::map_level::zoning::ZoningType;
use crate::resource_type::ResourceType;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::{IndustryBuildingId, PlayerId};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct IndustryBuildingInfo {
    id:            IndustryBuildingId,
    industry_type: IndustryType,
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
            self.industry_type,
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
        industry_type: IndustryType,
    ) -> Self {
        Self {
            id,
            industry_type,
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
    pub fn industry_type(&self) -> IndustryType {
        self.industry_type
    }

    #[must_use]
    pub fn reference_tile(&self) -> TileCoordsXZ {
        self.static_info.reference_tile()
    }

    pub fn advance_industry_building(&mut self, diff: GameTimeDiff) {
        let seconds = diff.to_seconds();
        self.advance_industry(seconds, self.industry_type());
    }

    #[must_use]
    pub fn required_zoning(&self) -> ZoningType {
        self.industry_type.required_zoning()
    }

    #[must_use]
    #[expect(clippy::match_same_arms)]
    pub fn industry_transform_inputs(&self) -> HashSet<ResourceType> {
        let mut result = HashSet::new();
        for input in self.industry_type.transform_per_second().inputs {
            result.insert(input.resource);
        }
        result
    }

    #[must_use]
    pub fn candidate_station_locations(&self) -> Vec<(TileCoordsXZ, StationType)> {
        let mut results = vec![];

        for z in -4 ..= 1 {
            results.push((TileCoordsXZ::new(-2, z), StationType::NS_1_4));
            results.push((TileCoordsXZ::new(2, z), StationType::NS_1_4));
        }

        for x in -4 ..= 1 {
            results.push((TileCoordsXZ::new(x, -2), StationType::EW_1_4));
            results.push((TileCoordsXZ::new(x, 2), StationType::EW_1_4));
        }

        results
            .into_iter()
            .map(|(tile, station_type)| (self.reference_tile() + tile, station_type))
            .collect()
    }

    #[must_use]
    pub fn industry_building_shippable_cargo(&self) -> CargoMap {
        let transform = self.industry_type.transform_per_second();
        let mut result = CargoMap::new();
        for output in transform.outputs {
            let resource = output.resource;
            // Later: This is now insta-shipping of everything... consider doing this more gradually?
            let amount = self.dynamic_info().cargo().get(resource);
            result.add(resource, amount);
        }
        result
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
        self.industry_type.relative_tiles_used()
    }
}

impl BuildingInfo for IndustryBuildingInfo {}

impl WithOwner for IndustryBuildingInfo {
    fn owner_id(&self) -> PlayerId {
        self.static_info.owner_id()
    }
}

impl WithTileCoverage for IndustryBuildingInfo {
    fn covers_tiles(&self) -> TileCoverage {
        self.relative_tiles_used().offset_by(self.reference_tile())
    }
}
