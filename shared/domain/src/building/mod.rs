#![allow(clippy::module_name_repetitions)]

use crate::tile_coverage::TileCoverage;

pub mod building_info;
pub mod building_state;
pub mod building_type;
pub mod industry_type;
pub mod station_type;
pub mod track_info;

pub trait WithRelativeTileCoverage {
    fn relative_tiles_used(&self) -> TileCoverage;
}
