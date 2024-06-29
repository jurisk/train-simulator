use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::building_type::BuildingType;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::{BuildingId, PlayerId};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct BuildingInfo {
    pub owner_id:       PlayerId,
    pub building_id:    BuildingId,
    pub reference_tile: TileCoordsXZ,
    pub building_type:  BuildingType,
}

impl BuildingInfo {
    #[must_use]
    pub fn covers_tiles(self) -> HashSet<TileCoordsXZ> {
        self.building_type
            .relative_tiles_used()
            .iter()
            .map(|relative_tile| self.reference_tile + *relative_tile)
            .collect()
    }
}
