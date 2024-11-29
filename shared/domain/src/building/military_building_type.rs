use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::building::WithRelativeTileCoverage;
use crate::building::building_info::WithCostToBuild;
use crate::building::industry_type::IndustryType;
use crate::cargo_map::CargoMap;
use crate::game_time::GameTimeDiff;
use crate::military::ProjectileType;
use crate::resource_type::ResourceType;
use crate::tile_coverage::TileCoverage;

// Later: Also movable units? Movable artillery? Troops? Trenches? Tanks? Ships? Airplanes?
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum MilitaryBuildingType {
    FixedArtillery,
}

impl Debug for MilitaryBuildingType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MilitaryBuildingType::FixedArtillery => write!(f, "FixedArtillery"),
        }
    }
}

impl MilitaryBuildingType {
    #[must_use]
    pub fn all() -> Vec<MilitaryBuildingType> {
        vec![MilitaryBuildingType::FixedArtillery]
    }

    #[must_use]
    pub fn reload_time(&self) -> GameTimeDiff {
        match self {
            MilitaryBuildingType::FixedArtillery => GameTimeDiff::from_seconds(20.0),
        }
    }

    #[must_use]
    pub fn projectile_type(&self) -> ProjectileType {
        match self {
            MilitaryBuildingType::FixedArtillery => ProjectileType::Standard,
        }
    }
}

impl WithRelativeTileCoverage for MilitaryBuildingType {
    fn relative_tiles_used(&self) -> TileCoverage {
        match self {
            MilitaryBuildingType::FixedArtillery => TileCoverage::single_at_zero(),
        }
    }
}

impl WithCostToBuild for MilitaryBuildingType {
    fn cost_to_build(&self) -> (IndustryType, CargoMap) {
        match self {
            MilitaryBuildingType::FixedArtillery => {
                (
                    IndustryType::MilitaryBase,
                    CargoMap::single(ResourceType::ArtilleryWeapons, 1.0f32),
                )
            },
        }
    }
}
