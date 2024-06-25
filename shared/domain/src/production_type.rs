use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::tile_coords_xz::TileCoordsXZ;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum ProductionType {
    CoalMine,
    IronMine,
    IronWorks,
}

impl ProductionType {
    #[must_use]
    pub(crate) fn relative_tiles_used(self) -> HashSet<TileCoordsXZ> {
        HashSet::new() // TODO: Implement
    }
}
