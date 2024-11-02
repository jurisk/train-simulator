use log::{debug, error, info, trace};
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::{IndustryBuildingId, PlayerId, StationId};

use crate::oct2025::GoalResult;
use crate::oct2025::stations::select_station_building;

#[derive(Clone, Debug)]
pub(crate) enum IndustryState {
    // TODO: Could have more gradual steps, e.g. don't assume that building will succeed and have "BuildingIndustry" and "BuildingStation" states...
    NothingDone,
    IndustryBuilt(IndustryBuildingId, TileCoordsXZ),
    StationBuilt(IndustryBuildingId, TileCoordsXZ, StationId),
}

impl IndustryState {
    #[expect(clippy::collapsible_else_if)]
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
                    *self = IndustryState::IndustryBuilt(building.id(), building.reference_tile());
                    GoalResult::SendCommands(vec![GameCommand::BuildIndustryBuilding(building)])
                } else {
                    info!(
                        "Failed to select building for {industry:?}, this could be normal if we lack resources"
                    );
                    GoalResult::Done
                }
            },
            IndustryState::IndustryBuilt(industry_building_id, location) => {
                if let Some(station) = game_state
                    .building_state()
                    .find_linked_station(*industry_building_id)
                {
                    *self =
                        IndustryState::StationBuilt(*industry_building_id, *location, station.id());
                    GoalResult::RepeatInvocation
                } else {
                    if let Some(building) = game_state
                        .building_state()
                        .find_industry_building(*industry_building_id)
                    {
                        let station = select_station_building(player_id, game_state, building);
                        trace!(
                            "Building station {station:?} for {industry:?} at {industry_building_id:?}"
                        );
                        if let Some(station) = station {
                            // TODO HIGH: Should be more gradual, we cannot declare it already built if we just sent the message
                            *self = IndustryState::StationBuilt(
                                *industry_building_id,
                                *location,
                                station.id(),
                            );
                            GoalResult::SendCommands(vec![GameCommand::BuildStation(station)])
                        } else {
                            // TODO: This could also be abnormal, as we may have built tracks in the neighbourhood before building all industries and stations, which prohibit building the station!
                            info!(
                                "Failed to select station for {industry:?} at {location:?}, this could be normal if we lack resources"
                            );
                            GoalResult::Done
                        }
                    } else {
                        error!(
                            "Industry building {industry_building_id:?} not found at {location:?}"
                        );
                        // TODO: This is a hack, but not sure how else to handle these weird situations - possibly race conditions.
                        *self = IndustryState::NothingDone;
                        GoalResult::RepeatInvocation
                    }
                }
            },
            IndustryState::StationBuilt(_industry_building_id, _location, _station_id) => {
                GoalResult::Done
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
    let free = game_state.all_free_zonings();

    let found = free
        .iter()
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
