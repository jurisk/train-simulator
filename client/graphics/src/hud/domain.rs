use bevy::prelude::Resource;
use shared_domain::building_type::BuildingType;
use shared_domain::industry_type::IndustryType;
use shared_domain::station_type::StationType;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::tile_coverage::TileCoverage;
use shared_domain::transport::transport_type::TransportType;
use shared_domain::TransportId;

#[derive(Resource, Eq, PartialEq, Debug, Clone)]
pub enum SelectedMode {
    Info,
    Tracks,
    Stations(StationType),
    Industry(IndustryType),
    Military,
    Transport(TransportType),
    Demolish,
    // Later: This feels like a hack, this is very much not like the others
    SelectStationToAppendToTransportMovementInstructions(TransportId),
}

impl SelectedMode {
    #[must_use]
    pub fn show_selected_edges(&self) -> bool {
        matches!(self, SelectedMode::Tracks)
    }

    #[must_use]
    pub fn show_hovered_edge(&self) -> bool {
        matches!(self, SelectedMode::Tracks) || matches!(self, SelectedMode::Transport(_))
    }

    #[must_use]
    pub fn show_selected_tiles(&self) -> bool {
        matches!(self, SelectedMode::Tracks)
    }

    #[must_use]
    pub fn show_hovered_tile(&self) -> bool {
        true
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn corresponding_building(&self) -> Option<BuildingType> {
        match self {
            SelectedMode::Tracks => None,
            SelectedMode::Stations(station_type) => Some(BuildingType::Station(*station_type)),
            SelectedMode::Industry(industry_type) => Some(BuildingType::Industry(*industry_type)),
            SelectedMode::Military => None,
            SelectedMode::Transport(_) => None,
            SelectedMode::Info => None,
            SelectedMode::Demolish => None,
            SelectedMode::SelectStationToAppendToTransportMovementInstructions(_) => None,
        }
    }

    #[must_use]
    pub fn building_tiles(&self, reference_tile: TileCoordsXZ) -> TileCoverage {
        match self.corresponding_building() {
            None => TileCoverage::Empty,
            Some(building) => building.relative_tiles_used().offset_by(reference_tile),
        }
    }
}
