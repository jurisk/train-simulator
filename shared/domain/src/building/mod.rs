#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::cargo_map::CargoMap;
use crate::tile_coverage::TileCoverage;
use crate::IndustryBuildingId;

pub mod building_info;
pub mod building_state;
pub mod industry_building_info;
pub mod industry_type;
pub mod station_info;
pub mod station_type;
pub mod track_info;
pub mod track_state;

pub trait WithRelativeTileCoverage {
    fn relative_tiles_used(&self) -> TileCoverage;
}

pub struct BuildCosts {
    pub costs: HashMap<IndustryBuildingId, CargoMap>,
}

impl BuildCosts {
    #[must_use]
    pub fn single(industry_building_id: IndustryBuildingId, cargo_map: CargoMap) -> Self {
        let mut costs = HashMap::new();
        costs.insert(industry_building_id, cargo_map);
        Self { costs }
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
