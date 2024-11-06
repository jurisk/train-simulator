use log::{debug, trace};
use shared_domain::building::industry_type::IndustryType;
use shared_domain::client_command::GameCommand;
use shared_domain::directional_edge::DirectionalEdge;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::resource_type::ResourceType;
use shared_domain::server_response::{GameError, GameResponse};
use shared_domain::transport::tile_track::TileTrack;
use shared_domain::transport::track_planner::{DEFAULT_ALREADY_EXISTS_COEF, plan_tracks};
use shared_domain::{PlayerId, TransportId};

use crate::oct2025::GoalResult;
use crate::oct2025::industries::IndustryState;
use crate::oct2025::stations::exit_tile_tracks;
use crate::oct2025::transports::purchase_transport;

#[derive(Clone)]
pub(crate) enum ResourceLinkState {
    Pending,
    TracksPending(Vec<(TileTrack, TileTrack)>),
    TracksBuilt,
    PurchasingTrain(TransportId),
    TrainPurchased,
}

fn track_pairs(
    game_state: &GameState,
    from_industry_state: &IndustryState,
    to_industry_state: &IndustryState,
) -> Option<Vec<(TileTrack, TileTrack)>> {
    let from_exit_tile_tracks = exit_tile_tracks(from_industry_state, game_state)?;
    let to_exit_tile_tracks = exit_tile_tracks(to_industry_state, game_state)?;

    let mut pairs = vec![];
    for track_a in &from_exit_tile_tracks {
        for track_b in &to_exit_tile_tracks {
            pairs.push((*track_a, *track_b));
            pairs.push((*track_b, *track_a));
        }
    }

    Some(pairs)
}

// TODO HIGH: This could be a Goal?
impl ResourceLinkState {
    pub(crate) fn notify_of_response(&mut self, response: &GameResponse) {
        if let GameResponse::Error(error) = response {
            match error {
                GameError::CannotPurchaseTransport(transport_id, _) => {
                    if let ResourceLinkState::PurchasingTrain(pending_transport_id) = self {
                        if *pending_transport_id == *transport_id {
                            *self = ResourceLinkState::TracksBuilt;
                        }
                    }
                },
                GameError::CannotBuildTracks(..) => {
                    if let ResourceLinkState::TracksPending(_) = self {
                        // This is somewhat questionable, as on any error we are going back to square one, and also we might be getting events unrelated to our particular resource link... but the alternative is adding some "TrackBuildingRequestId" and correlating that, and that is adding complexity.
                        *self = ResourceLinkState::Pending;
                    }
                },
                _ => {},
            }
        }
    }

    #[expect(clippy::collapsible_else_if)]
    #[must_use]
    pub(crate) fn commands(
        &mut self,
        from_industry_state: &IndustryState,
        resource: ResourceType,
        to_industry_state: &IndustryState,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> GoalResult {
        match self {
            ResourceLinkState::Pending => {
                match track_pairs(game_state, from_industry_state, to_industry_state) {
                    Some(pairs) => {
                        *self = ResourceLinkState::TracksPending(pairs);
                        GoalResult::RepeatInvocation
                    },
                    None => {
                        // This resource link is not ready yet to be planned
                        GoalResult::TryAgainLater
                    },
                }
            },
            ResourceLinkState::TracksPending(pairs) => {
                if let Some((source, target)) = pairs.pop() {
                    if let Some((route, _length)) = plan_tracks(
                        player_id,
                        DirectionalEdge::exit_from(source),
                        &[DirectionalEdge::entrance_to(target)],
                        game_state,
                        DEFAULT_ALREADY_EXISTS_COEF,
                        metrics,
                    ) {
                        if route.is_empty() {
                            // If it's empty, it means it's already built
                            GoalResult::RepeatInvocation
                        } else {
                            if game_state.can_build_tracks(player_id, &route).is_ok() {
                                GoalResult::SendCommands(vec![GameCommand::BuildTracks(route)])
                            } else {
                                GoalResult::SendCommands(vec![])
                            }
                        }
                    } else {
                        debug!("No route found for {:?} -> {:?}", source, target);
                        GoalResult::RepeatInvocation
                    }
                } else {
                    *self = ResourceLinkState::TracksBuilt;
                    GoalResult::RepeatInvocation
                }
            },
            ResourceLinkState::TracksBuilt => {
                // TODO: Buy more transports if the tracks are long, or perhaps if a backlog of resources gets formed
                // TODO HIGH: Ammunition train was missing when testing!
                if let Some((station, transport)) = purchase_transport(
                    player_id,
                    game_state,
                    from_industry_state,
                    resource,
                    to_industry_state,
                ) {
                    let transport_id = transport.transport_id();
                    *self = ResourceLinkState::PurchasingTrain(transport_id);

                    let command = GameCommand::PurchaseTransport(station, transport);
                    GoalResult::SendCommands(vec![command])
                } else {
                    trace!(
                        "Failed to purchase transport for {resource:?}, this could be normal if we lack resources"
                    );
                    GoalResult::TryAgainLater
                }
            },
            ResourceLinkState::PurchasingTrain(transport_id) => {
                if game_state
                    .transport_state()
                    .find_players_transport(player_id, *transport_id)
                    .is_some()
                {
                    *self = ResourceLinkState::TrainPurchased;
                }

                GoalResult::RepeatInvocation
            },
            ResourceLinkState::TrainPurchased => GoalResult::Finished,
        }
    }
}

pub(crate) fn resource_links(
    industries: &[IndustryType],
) -> Vec<(IndustryType, ResourceType, IndustryType)> {
    let mut results = vec![];
    for a in industries {
        for b in industries {
            for c in ResourceType::all() {
                if a.produces(c) && b.consumes(c) {
                    results.push((*a, c, *b));
                }
            }
        }
    }
    results
}
