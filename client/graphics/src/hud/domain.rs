use bevy::prelude::Resource;
use shared_domain::building_type::BuildingType;
use shared_domain::production_type::ProductionType;
use shared_domain::station_type::{StationOrientation, StationType};
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::tile_coverage::TileCoverage;

#[derive(Resource, Eq, PartialEq, Debug, Copy, Clone)]
pub enum SelectedMode {
    Info,
    Tracks,
    Stations(StationOrientation),
    Production(ProductionType),
    Military,
    Trains,
    Demolish,
}

impl SelectedMode {
    #[must_use]
    pub fn show_selected_edges(self) -> bool {
        matches!(self, SelectedMode::Tracks)
    }

    #[must_use]
    pub fn show_hovered_edge(self) -> bool {
        matches!(self, SelectedMode::Tracks)
    }

    #[must_use]
    pub fn show_selected_tiles(self) -> bool {
        matches!(self, SelectedMode::Tracks)
    }

    #[must_use]
    pub fn show_hovered_tile(self) -> bool {
        true
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn corresponding_building(self) -> Option<BuildingType> {
        match self {
            SelectedMode::Tracks => None,
            SelectedMode::Stations(orientation) => {
                Some(BuildingType::Station(StationType {
                    orientation,
                    platforms: 1,
                    length_in_tiles: 4,
                }))
            },
            SelectedMode::Production(production_type) => {
                Some(BuildingType::Production(production_type))
            },
            SelectedMode::Military => None,
            SelectedMode::Trains => None,
            SelectedMode::Info => None,
            SelectedMode::Demolish => None,
        }
    }

    #[must_use]
    pub fn building_tiles(self, reference_tile: TileCoordsXZ) -> TileCoverage {
        match self.corresponding_building() {
            None => TileCoverage::Empty,
            Some(building) => building.relative_tiles_used().offset_by(reference_tile),
        }
    }
}
