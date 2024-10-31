use log::{debug, error};
use shared_domain::building::industry_type::IndustryType;
use shared_domain::client_command::GameCommand;
use shared_domain::directional_edge::DirectionalEdge;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::resource_type::ResourceType;
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

impl ResourceLinkState {
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
                let from_exit_tile_tracks = exit_tile_tracks(from_industry_state, game_state);
                let to_exit_tile_tracks = exit_tile_tracks(to_industry_state, game_state);

                let mut pairs = vec![];
                for track_a in &from_exit_tile_tracks {
                    for track_b in &to_exit_tile_tracks {
                        pairs.push((*track_a, *track_b));
                        pairs.push((*track_b, *track_a));
                    }
                }
                *self = ResourceLinkState::TracksPending(pairs);
                GoalResult::RepeatInvocation
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
                let command = purchase_transport(
                    player_id,
                    game_state,
                    from_industry_state,
                    resource,
                    to_industry_state,
                );

                if let Some(ref command @ GameCommand::PurchaseTransport(_, ref transport)) =
                    command
                {
                    let transport_id = transport.transport_id();
                    *self = ResourceLinkState::PurchasingTrain(transport_id);
                    GoalResult::SendCommands(vec![command.clone()])
                } else {
                    error!("Unexpected command for creating transport: {command:?}");
                    GoalResult::Done
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

                GoalResult::Done
            },
            ResourceLinkState::TrainPurchased => GoalResult::Done,
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
