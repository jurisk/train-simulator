use log::{debug, error, trace};
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::server_response::{GameError, GameResponse};
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::{IndustryBuildingId, PlayerId, StationId};

use crate::oct2025::stations::select_station_building;
use crate::oct2025::{Goal, GoalResult};

#[derive(Clone, Debug)]
pub(crate) struct BuildIndustry {
    pub(crate) industry_type:   IndustryType,
    pub(crate) target_location: TileCoordsXZ,
    pub(crate) state:           BuildIndustryState,
}

#[derive(Clone, Debug)]
pub(crate) enum BuildIndustryState {
    NothingDone,
    BuildingIndustry(IndustryBuildingId, TileCoordsXZ),
    IndustryBuilt(IndustryBuildingId, TileCoordsXZ),
    BuildingStation(IndustryBuildingId, TileCoordsXZ, StationId),
    StationBuilt(IndustryBuildingId, TileCoordsXZ, StationId),
}

impl Goal for BuildIndustry {
    #[must_use]
    #[expect(clippy::too_many_lines)]
    fn commands(
        &mut self,
        player_id: PlayerId,
        game_state: &GameState,
        _metrics: &dyn Metrics,
    ) -> GoalResult {
        trace!("BuildIndustry for {self:?}");
        match self.state {
            BuildIndustryState::NothingDone => {
                if let Some(building) = select_industry_building(
                    player_id,
                    game_state,
                    self.industry_type,
                    self.target_location,
                ) {
                    self.state = BuildIndustryState::BuildingIndustry(
                        building.id(),
                        building.reference_tile(),
                    );
                    GoalResult::SendCommands(vec![GameCommand::BuildIndustryBuilding(building)])
                } else {
                    trace!(
                        "Failed to select building for {:?}, this could be normal if we lack resources",
                        self.industry_type
                    );
                    GoalResult::TryAgainLater
                }
            },
            BuildIndustryState::BuildingIndustry(industry_building_id, location) => {
                if let Some(industry) = game_state
                    .building_state()
                    .find_industry_building(industry_building_id)
                {
                    if industry.id() == industry_building_id
                        && industry.reference_tile() == location
                    {
                        self.state =
                            BuildIndustryState::IndustryBuilt(industry_building_id, location);
                        GoalResult::RepeatInvocation
                    } else {
                        let message = format!(
                            "Industry building {industry_building_id:?} not found at {location:?}"
                        );
                        error!("{message}");
                        GoalResult::Error(message)
                    }
                } else {
                    GoalResult::TryAgainLater
                }
            },
            BuildIndustryState::IndustryBuilt(industry_building_id, location) => {
                if let Some(building) = game_state
                    .building_state()
                    .find_industry_building(industry_building_id)
                {
                    if let Some(station) = game_state
                        .building_state()
                        .find_linked_station(industry_building_id)
                    {
                        self.state = BuildIndustryState::StationBuilt(
                            industry_building_id,
                            location,
                            station.id(),
                        );
                        GoalResult::Finished
                    } else {
                        let station = select_station_building(player_id, game_state, building);
                        trace!(
                            "Building station {station:?} for {:?} at {industry_building_id:?}",
                            self.industry_type
                        );
                        if let Some(station) = station {
                            self.state = BuildIndustryState::BuildingStation(
                                industry_building_id,
                                location,
                                station.id(),
                            );
                            GoalResult::SendCommands(vec![GameCommand::BuildStation(station)])
                        } else {
                            // TODO: This could also be abnormal, as we may have built tracks in the neighbourhood before building all industries and stations, which prohibit building the station!
                            trace!(
                                "Failed to select station for {:?} at {location:?}, this could be normal if we lack resources",
                                self.industry_type
                            );
                            GoalResult::TryAgainLater
                        }
                    }
                } else {
                    let message = format!(
                        "Industry building {industry_building_id:?} not found at {location:?}"
                    );
                    error!("{message}");
                    // TODO: This is a hack, but not sure how else to handle these weird situations - possibly race conditions.
                    self.state = BuildIndustryState::NothingDone;
                    GoalResult::Error(message)
                }
            },
            BuildIndustryState::BuildingStation(industry_building_id, location, _station_id) => {
                if let Some(station) = game_state
                    .building_state()
                    .find_linked_station(industry_building_id)
                {
                    self.state = BuildIndustryState::StationBuilt(
                        industry_building_id,
                        location,
                        station.id(),
                    );
                    GoalResult::Finished
                } else {
                    GoalResult::TryAgainLater
                }
            },
            BuildIndustryState::StationBuilt(_industry_building_id, _location, _station_id) => {
                GoalResult::Finished
            },
        }
    }

    fn notify_of_response(&mut self, response: &GameResponse) {
        if let GameResponse::Error(game_error) = response {
            match game_error {
                GameError::CannotBuildStation(that_station_id, build_error) => {
                    if let BuildIndustryState::BuildingStation(
                        industry_building_id,
                        location,
                        station_id,
                    ) = self.state
                    {
                        if station_id == *that_station_id {
                            error!(
                                "Failed to build station {station_id:?} for industry at {location:?}: {build_error:?}, rolling back"
                            );
                            self.state =
                                BuildIndustryState::IndustryBuilt(industry_building_id, location);
                        }
                    }
                },
                GameError::CannotBuildIndustryBuilding(that_industry_id, build_error) => {
                    if let BuildIndustryState::BuildingIndustry(industry_building_id, location) =
                        self.state
                    {
                        if industry_building_id == *that_industry_id {
                            error!(
                                "Failed to build industry {industry_building_id:?} at {location:?}: {build_error:?}, rolling back"
                            );
                            self.state = BuildIndustryState::NothingDone;
                        }
                    }
                },
                _ => {},
            }
        }
    }
}

pub(crate) fn select_industry_building(
    owner_id: PlayerId,
    game_state: &GameState,
    industry_type: IndustryType,
    reference_tile: TileCoordsXZ,
) -> Option<IndustryBuildingInfo> {
    let mut zonings: Vec<_> = game_state
        .all_free_zonings()
        .filter(|zoning| Some(zoning.zoning_type()) == industry_type.required_zoning())
        .collect();

    // TODO: Actually, build close to related industries in this supply chain - I think this 'reference_tile' is for the main building
    zonings.sort_by_key(|zoning| zoning.reference_tile().manhattan_distance(reference_tile));

    let found = zonings
        .into_iter()
        .map(|zoning| {
            IndustryBuildingInfo::new(
                owner_id,
                IndustryBuildingId::random(),
                zoning.reference_tile(),
                industry_type,
            )
        })
        .find(|info| {
            game_state
                .can_build_industry_building(owner_id, info)
                .is_ok()
        });

    // TODO: If industry has no zoning requirement, build in an empty space, but choose the best place - closest to the industries for its inputs/outputs, or even just closest to ConstructionYard.
    if let Some(info) = found {
        Some(info.clone())
    } else {
        debug!("No free zoning for {:?}", industry_type);
        None
    }
}
