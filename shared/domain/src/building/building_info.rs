use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::building::industry_type::IndustryType;
use crate::cargo_map::{CargoMap, WithCargo, WithCargoMut};
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::PlayerId;

pub trait BuildingInfo: WithOwner + WithTileCoverage {}

pub trait WithOwner {
    fn owner_id(&self) -> PlayerId;
}

pub trait WithTileCoverage {
    fn covers_tiles(&self) -> TileCoverage;
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct BuildingStaticInfo {
    owner_id:       PlayerId,
    reference_tile: TileCoordsXZ,
}

impl BuildingStaticInfo {
    #[must_use]
    pub fn new(owner_id: PlayerId, reference_tile: TileCoordsXZ) -> Self {
        Self {
            owner_id,
            reference_tile,
        }
    }

    #[must_use]
    pub fn owner_id(&self) -> PlayerId {
        self.owner_id
    }

    #[must_use]
    pub fn reference_tile(&self) -> TileCoordsXZ {
        self.reference_tile
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct BuildingDynamicInfo {
    cargo: CargoMap,
}

impl BuildingDynamicInfo {
    #[must_use]
    pub fn new(cargo: CargoMap) -> Self {
        Self { cargo }
    }
}

impl WithCargo for &BuildingDynamicInfo {
    fn cargo(&self) -> &CargoMap {
        &self.cargo
    }
}

impl WithCargoMut for &mut BuildingDynamicInfo {
    fn cargo_mut(&mut self) -> &mut CargoMap {
        &mut self.cargo
    }
}

impl Debug for BuildingDynamicInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.cargo)
    }
}

pub trait WithBuildingDynamicInfo {
    fn dynamic_info(&self) -> &BuildingDynamicInfo;
}

pub trait WithBuildingDynamicInfoMut {
    fn dynamic_info_mut(&mut self) -> &mut BuildingDynamicInfo;
}

impl<T: WithBuildingDynamicInfo> WithCargo for T {
    fn cargo(&self) -> &CargoMap {
        &self.dynamic_info().cargo
    }
}

impl<T: WithBuildingDynamicInfoMut> WithCargoMut for T {
    fn cargo_mut(&mut self) -> &mut CargoMap {
        &mut self.dynamic_info_mut().cargo
    }
}

pub trait WithCostToBuild {
    /// Returns what building is needed in supply range to build this building, and how many resources it requires.
    fn cost_to_build(self) -> (IndustryType, CargoMap);
}
