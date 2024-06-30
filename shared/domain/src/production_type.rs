use serde::{Deserialize, Serialize};

use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum ProductionType {
    CoalMine,
    IronMine,
    IronWorks,
    CargoPort,
}

impl ProductionType {
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [
            ProductionType::CoalMine,
            ProductionType::IronMine,
            ProductionType::IronWorks,
            ProductionType::CargoPort,
        ]
    }

    #[must_use]
    pub(crate) fn relative_tiles_used(self) -> TileCoverage {
        TileCoverage::Rectangular {
            north_west_inclusive: TileCoordsXZ::new(-1, -1),
            south_east_inclusive: TileCoordsXZ::new(1, 1),
        }
    }
}
