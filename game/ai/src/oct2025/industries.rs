use log::{debug, error, trace};
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::server_response::{GameError, GameResponse};
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::{IndustryBuildingId, PlayerId, StationId};

use crate::oct2025::GoalResult;
use crate::oct2025::stations::select_station_building;

#[derive(Clone, Debug)]
pub(crate) enum IndustryState {
    NothingDone,
    BuildingIndustry(IndustryBuildingId, TileCoordsXZ),
    IndustryBuilt(IndustryBuildingId, TileCoordsXZ),
    BuildingStation(IndustryBuildingId, TileCoordsXZ, StationId),
    StationBuilt(IndustryBuildingId, TileCoordsXZ, StationId),
}

// TODO HIGH: Should be a 'Goal' probably?
impl IndustryState {
    pub(crate) fn notify_of_response(&mut self, response: &GameResponse) {
        if let GameResponse::Error(game_error) = response {
            match game_error {
                GameError::CannotBuildStation(that_station_id, build_error) => {
                    if let IndustryState::BuildingStation(
                        industry_building_id,
                        location,
                        station_id,
                    ) = self
                    {
                        if *station_id == *that_station_id {
                            error!(
                                "Failed to build station {station_id:?} for industry at {location:?}: {build_error:?}, rolling back"
                            );
                            *self = IndustryState::IndustryBuilt(*industry_building_id, *location);
                        }
                    }
                },
                GameError::CannotBuildIndustryBuilding(that_industry_id, build_error) => {
                    if let IndustryState::BuildingIndustry(industry_building_id, location) = self {
                        if *industry_building_id == *that_industry_id {
                            error!(
                                "Failed to build industry {industry_building_id:?} at {location:?}: {build_error:?}, rolling back"
                            );
                            *self = IndustryState::NothingDone;
                        }
                    }
                },
                _ => {},
            }
        }
    }

    #[must_use]
    pub(crate) fn commands(
        &mut self,
        industry: IndustryType,
        player_id: PlayerId,
        game_state: &GameState,
        target_location: TileCoordsXZ,
    ) -> GoalResult {
        trace!("IndustryState for {industry:?}: {self:?}");
        match self {
            IndustryState::NothingDone => {
                if let Some(building) =
                    select_industry_building(player_id, game_state, industry, target_location)
                {
                    *self =
                        IndustryState::BuildingIndustry(building.id(), building.reference_tile());
                    GoalResult::SendCommands(vec![GameCommand::BuildIndustryBuilding(building)])
                } else {
                    trace!(
                        "Failed to select building for {industry:?}, this could be normal if we lack resources"
                    );
                    GoalResult::TryAgainLater
                }
            },
            IndustryState::BuildingIndustry(industry_building_id, location) => {
                if let Some(industry) = game_state
                    .building_state()
                    .find_industry_building(*industry_building_id)
                {
                    if industry.id() == *industry_building_id
                        && industry.reference_tile() == *location
                    {
                        *self = IndustryState::IndustryBuilt(*industry_building_id, *location);
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
            IndustryState::IndustryBuilt(industry_building_id, location) => {
                if let Some(building) = game_state
                    .building_state()
                    .find_industry_building(*industry_building_id)
                {
                    if let Some(station) = game_state
                        .building_state()
                        .find_linked_station(*industry_building_id)
                    {
                        *self = IndustryState::StationBuilt(
                            *industry_building_id,
                            *location,
                            station.id(),
                        );
                        GoalResult::Finished
                    } else {
                        let station = select_station_building(player_id, game_state, building);
                        trace!(
                            "Building station {station:?} for {industry:?} at {industry_building_id:?}"
                        );
                        if let Some(station) = station {
                            *self = IndustryState::BuildingStation(
                                *industry_building_id,
                                *location,
                                station.id(),
                            );
                            GoalResult::SendCommands(vec![GameCommand::BuildStation(station)])
                        } else {
                            // TODO: This could also be abnormal, as we may have built tracks in the neighbourhood before building all industries and stations, which prohibit building the station!
                            trace!(
                                "Failed to select station for {industry:?} at {location:?}, this could be normal if we lack resources"
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
                    *self = IndustryState::NothingDone;
                    GoalResult::Error(message)
                }
            },
            IndustryState::BuildingStation(industry_building_id, location, _station_id) => {
                if let Some(station) = game_state
                    .building_state()
                    .find_linked_station(*industry_building_id)
                {
                    *self =
                        IndustryState::StationBuilt(*industry_building_id, *location, station.id());
                    GoalResult::Finished
                } else {
                    GoalResult::TryAgainLater
                }
            },
            IndustryState::StationBuilt(_industry_building_id, _location, _station_id) => {
                GoalResult::Finished
            },
        }
    }
}

pub(crate) fn select_industry_building(
    owner_id: PlayerId,
    game_state: &GameState,
    industry_type: IndustryType,
    reference_tile: TileCoordsXZ,
) -> Option<IndustryBuildingInfo> {
    let found = game_state
        .all_free_zonings()
        .filter(|zoning| Some(zoning.zoning_type()) == industry_type.required_zoning())
        .map(|zoning| {
            IndustryBuildingInfo::new(
                owner_id,
                IndustryBuildingId::random(),
                zoning.reference_tile(),
                industry_type,
            )
        })
        .filter(|info| {
            game_state
                .can_build_industry_building(owner_id, info)
                .is_ok()
        })
        .min_by_key(|info| {
            // TODO: Actually, build close to related industries in this supply chain
            info.reference_tile().manhattan_distance(reference_tile)
        });

    // TODO: If industry has no zoning requirement, build in an empty space, but choose the best place - closest to the industries for its inputs/outputs, or even just closest to ConstructionYard.
    if let Some(info) = found {
        Some(info.clone())
    } else {
        debug!("No free zoning for {:?}", industry_type);
        None
    }
}
