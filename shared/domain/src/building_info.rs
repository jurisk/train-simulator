use serde::{Deserialize, Serialize};

use crate::building_type::BuildingType;
use crate::tile_coverage::TileCoverage;
use crate::{BuildingId, PlayerId};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct BuildingInfo {
    pub owner_id:      PlayerId,
    pub building_id:   BuildingId,
    pub covers_tiles:  TileCoverage,
    pub building_type: BuildingType,
}
