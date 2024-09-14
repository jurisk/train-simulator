use bevy::prelude::Resource;
use shared_domain::building::building_info::WithTileCoverage;
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::station_info::StationInfo;
use shared_domain::building::station_type::StationType;
use shared_domain::client_command::GameCommand;
use shared_domain::directional_edge::DirectionalEdge;
use shared_domain::game_state::GameState;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::tile_coverage::TileCoverage;
use shared_domain::transport::transport_type::TransportType;
use shared_domain::{IndustryBuildingId, PlayerId, StationId, TransportId};

#[derive(Resource, Eq, PartialEq, Debug, Clone, Copy)]
pub enum DemolishType {
    Industry,
    Station,
    Tracks,
}

#[derive(Resource, Eq, PartialEq, Debug, Clone, Copy)]
pub enum SelectType {
    StationToAppendToTransportMovementInstructions(TransportId),
}

#[derive(Resource, Eq, PartialEq, Debug, Clone)]
pub enum TracksBuildingType {
    SelectStart,
    SelectEnd { start: DirectionalEdge },
}

// Later: Structure this more logically, it's too flattened
#[derive(Resource, Eq, PartialEq, Debug, Clone)]
pub enum SelectedMode {
    Info,
    Tracks(TracksBuildingType),
    Stations(StationType),
    Industry(IndustryType),
    Military,
    Transport(TransportType),
    Demolish(DemolishType),
    // Later: This feels like a hack, this is very much not like the others
    Select(SelectType),
}

// Later: Since we are no longer showing selected edges or tiles in any mode, we could simplify this. But think carefully - you may need it later.
impl SelectedMode {
    #[must_use]
    pub fn show_selected_edges(&self) -> bool {
        false
    }

    #[must_use]
    pub fn show_hovered_edge(&self) -> bool {
        matches!(self, SelectedMode::Tracks(_)) || matches!(self, SelectedMode::Transport(_))
    }

    #[must_use]
    pub fn show_selected_tiles(&self) -> bool {
        false
    }

    #[must_use]
    pub fn show_hovered_tile(&self) -> bool {
        true
    }

    #[must_use]
    pub fn build_building_command(
        &self,
        player_id: PlayerId,
        tile: TileCoordsXZ,
    ) -> Option<GameCommand> {
        match self {
            SelectedMode::Stations(station_type) => {
                Some(GameCommand::BuildStation(StationInfo::new(
                    player_id,
                    StationId::random(),
                    tile,
                    *station_type,
                )))
            },
            SelectedMode::Industry(industry_type) => {
                Some(GameCommand::BuildIndustryBuilding(
                    IndustryBuildingInfo::new(
                        player_id,
                        IndustryBuildingId::random(),
                        tile,
                        *industry_type,
                    ),
                ))
            },
            _ => None,
        }
    }

    #[must_use]
    pub fn building_tiles(
        &self,
        reference_tile: TileCoordsXZ,
        player_id: PlayerId,
        game_state: &GameState,
    ) -> Option<(TileCoverage, bool)> {
        match self.build_building_command(player_id, reference_tile) {
            Some(GameCommand::BuildStation(station_info)) => {
                Some((
                    station_info.covers_tiles(),
                    game_state.can_build_station(player_id, &station_info),
                ))
            },
            Some(GameCommand::BuildIndustryBuilding(industry_info)) => {
                Some((
                    industry_info.covers_tiles(),
                    game_state.can_build_industry_building(player_id, &industry_info),
                ))
            },
            _ => None,
        }
    }
}
