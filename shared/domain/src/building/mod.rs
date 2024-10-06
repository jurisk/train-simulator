#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;
use std::ops::AddAssign;

use serde::{Deserialize, Serialize};

use crate::IndustryBuildingId;
use crate::cargo_map::CargoMap;
use crate::tile_coverage::TileCoverage;

pub mod building_info;
pub mod building_state;
pub mod industry_building_info;
pub mod industry_type;
pub mod military_building_info;
pub mod military_building_type;
pub mod station_info;
pub mod station_type;
pub mod track_info;
pub mod track_state;

pub trait WithRelativeTileCoverage {
    fn relative_tiles_used(&self) -> TileCoverage;
}

#[derive(PartialEq, Clone, Debug)]
pub struct BuildCosts {
    pub costs: HashMap<IndustryBuildingId, CargoMap>,
}

impl BuildCosts {
    #[must_use]
    pub fn none() -> Self {
        Self {
            costs: HashMap::new(),
        }
    }

    #[must_use]
    pub fn single(industry_building_id: IndustryBuildingId, cargo_map: CargoMap) -> Self {
        let mut costs = HashMap::new();
        costs.insert(industry_building_id, cargo_map);
        Self { costs }
    }
}

impl AddAssign for BuildCosts {
    fn add_assign(&mut self, rhs: Self) {
        for (industry_building_id, cargo_map) in rhs.costs {
            self.costs
                .entry(industry_building_id)
                .and_modify(|existing| *existing += &cargo_map)
                .or_insert(cargo_map);
        }
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Copy, Clone, Debug)]
pub enum BuildError {
    InvalidOverlap,
    InvalidTerrain,
    InvalidZoning,
    NotEnoughResources,
    InvalidOwner,
    UnknownError,
}
