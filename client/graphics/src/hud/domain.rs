use bevy::prelude::Resource;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::station_type::StationType;
use shared_domain::building::WithRelativeTileCoverage;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::tile_coverage::TileCoverage;
use shared_domain::transport::transport_type::TransportType;
use shared_domain::TransportId;

// Later: Structure this more logically, it's too flattened
#[derive(Resource, Eq, PartialEq, Debug, Clone)]
pub enum SelectedMode {
    Info,
    Tracks,
    Stations(StationType),
    Industry(IndustryType),
    Military,
    Transport(TransportType),
    DemolishStation,
    DemolishIndustry,
    DemolishTracks,
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
    pub fn building_tiles(&self, reference_tile: TileCoordsXZ) -> TileCoverage {
        let relative_tiles = match self {
            SelectedMode::Stations(station_type) => Some(station_type.relative_tiles_used()),
            SelectedMode::Industry(industry_type) => Some(industry_type.relative_tiles_used()),
            _ => None,
        };

        relative_tiles.map_or(TileCoverage::Empty, |tiles| tiles.offset_by(reference_tile))
    }
}
