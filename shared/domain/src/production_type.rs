use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::tile_coords_xz::TileCoordsXZ;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum ProductionType {
    CoalMine,
    IronMine,
    IronWorks,
    CargoPort,
}

impl ProductionType {
    #[must_use]
    pub(crate) fn relative_tiles_used(self) -> HashSet<TileCoordsXZ> {
        let mut result = HashSet::new();
        for x in -1 ..= 1 {
            for z in -1 ..= 1 {
                result.insert(TileCoordsXZ::new(x, z));
            }
        }
        result
    }
}
