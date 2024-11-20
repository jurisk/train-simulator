use std::collections::{HashMap, HashSet};

use log::{error, trace};
use shared_domain::building::industry_type::IndustryType;
use shared_domain::client_command::GameCommand;
use shared_domain::directional_edge::DirectionalEdge;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::resource_type::ResourceType;
use shared_domain::server_response::{GameError, GameResponse};
use shared_domain::transport::tile_track::TileTrack;
use shared_domain::transport::track_length::TrackLength;
use shared_domain::transport::track_planner::{DEFAULT_ALREADY_EXISTS_COEF, plan_tracks};
use shared_domain::{PlayerId, StationId, TransportId};

use crate::oct2025::GoalResult;
use crate::oct2025::stations::exit_tile_tracks;
use crate::oct2025::transports::purchase_transport;

#[derive(Clone, Debug)]
pub(crate) struct BuildResourceLink {
    pub(crate) from_station_id: StationId,
    pub(crate) resource:        ResourceType,
    pub(crate) to_station_id:   StationId,
    pub(crate) state:           ResourceLinkState,
}

#[derive(Clone, Debug)]
pub(crate) enum ResourceLinkState {
    Pending,
    BuildingTracks {
        tracks_pending: Vec<(TileTrack, TileTrack)>,
        tracks_built:   HashMap<(TileTrack, TileTrack), TrackLength>,
    },
    TracksBuilt(HashMap<(TileTrack, TileTrack), TrackLength>),
    PurchasingTrains {
        target_trains:     usize,
        purchasing_trains: HashSet<TransportId>,
        purchased_trains:  HashSet<TransportId>,
    },
    TrainsPurchased,
}

fn track_pairs(
    game_state: &GameState,
    from_station_id: StationId,
    to_station_id: StationId,
) -> Option<Vec<(TileTrack, TileTrack)>> {
    let from_exit_tile_tracks = exit_tile_tracks(from_station_id, game_state)?;
    let to_exit_tile_tracks = exit_tile_tracks(to_station_id, game_state)?;

    let mut pairs = vec![];
    for track_a in &from_exit_tile_tracks {
        for track_b in &to_exit_tile_tracks {
            pairs.push((*track_a, *track_b));
            pairs.push((*track_b, *track_a));
        }
    }

    Some(pairs)
}

// We wanted this to be a `Goal` but it was not trivial to achieve
impl BuildResourceLink {
    pub(crate) fn notify_of_response(&mut self, response: &GameResponse) {
        match response {
            GameResponse::TransportsAdded(transports) => {
                if let ResourceLinkState::PurchasingTrains {
                    purchasing_trains,
                    purchased_trains,
                    ..
                } = &mut self.state
                {
                    for transport in transports {
                        if purchasing_trains.contains(&transport.transport_id()) {
                            purchasing_trains.remove(&transport.transport_id());
                            purchased_trains.insert(transport.transport_id());
                        }
                    }
                }
            },
            GameResponse::Error(error) => {
                match error {
                    GameError::CannotPurchaseTransport(transport_id, _) => {
                        if let ResourceLinkState::PurchasingTrains {
                            purchasing_trains, ..
                        } = &mut self.state
                        {
                            if purchasing_trains.contains(transport_id) {
                                purchasing_trains.remove(transport_id);
                            }
                        }
                    },
                    GameError::CannotBuildTracks(..) => {
                        if let ResourceLinkState::BuildingTracks { .. } = &self.state {
                            // This is somewhat questionable, as on any error we are going back to square one, and also we might be getting events unrelated to our particular resource link... but the alternative is adding some "TrackBuildingRequestId" and correlating that, and that is adding complexity.
                            self.state = ResourceLinkState::Pending;
                        }
                    },
                    _ => {},
                }
            },
            _ => {},
        }
    }

    #[expect(
        clippy::collapsible_else_if,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    #[must_use]
    pub(crate) fn commands(
        &mut self,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> GoalResult {
        match &mut self.state {
            ResourceLinkState::Pending => {
                match track_pairs(game_state, self.from_station_id, self.to_station_id) {
                    Some(pairs) => {
                        self.state = ResourceLinkState::BuildingTracks {
                            tracks_pending: pairs,
                            tracks_built:   HashMap::new(),
                        };
                        GoalResult::RepeatInvocation
                    },
                    None => {
                        // This resource link is not ready yet to be planned
                        GoalResult::TryAgainLater
                    },
                }
            },
            ResourceLinkState::BuildingTracks {
                tracks_pending,
                tracks_built,
            } => {
                if let Some((source, target)) = tracks_pending.pop() {
                    // TODO HIGH: We still fail to sometimes build tracks... Even if we can build them later. Perhaps we should only consider the tracks as built when we have confirmed a route exists?
                    if let Some((route, length)) = plan_tracks(
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
                                tracks_built.insert((source, target), length);
                                GoalResult::SendCommands(vec![GameCommand::BuildTracks(route)])
                            } else {
                                GoalResult::SendCommands(vec![])
                            }
                        }
                    } else {
                        // TODO HIGH: This is actually bad. This is possibly a blocked station or something else bad. And this current implementation will lead to an infinite loop.
                        error!("Failed building a route for {source:?} -> {target:?}");
                        // Returning the popped pair, we will try again...
                        tracks_pending.push((source, target));
                        GoalResult::TryAgainLater
                    }
                } else {
                    self.state = ResourceLinkState::TracksBuilt(tracks_built.clone());
                    GoalResult::RepeatInvocation
                }
            },
            ResourceLinkState::TracksBuilt(tracks_built) => {
                const TRAINS_PER_LENGTH_COEF: f32 = 0.01;

                let total_length = tracks_built.values().copied().sum::<TrackLength>();

                let target_trains = (total_length.to_f32() * TRAINS_PER_LENGTH_COEF)
                    .ceil()
                    .max(1f32) as usize;

                trace!("Total length: {total_length:?}, target_trains: {target_trains:?}");

                self.state = ResourceLinkState::PurchasingTrains {
                    target_trains,
                    purchasing_trains: HashSet::new(),
                    purchased_trains: HashSet::new(),
                };
                GoalResult::RepeatInvocation
            },
            ResourceLinkState::PurchasingTrains {
                target_trains,
                purchasing_trains,
                purchased_trains,
            } => {
                if purchased_trains.len() >= *target_trains && purchasing_trains.is_empty() {
                    self.state = ResourceLinkState::TrainsPurchased;
                    GoalResult::RepeatInvocation
                } else {
                    if let Some((station, transport)) = purchase_transport(
                        player_id,
                        game_state,
                        self.from_station_id,
                        self.resource,
                        self.to_station_id,
                    ) {
                        let transport_id = transport.transport_id();
                        purchasing_trains.insert(transport_id);

                        let command = GameCommand::PurchaseTransport(station, transport);
                        GoalResult::SendCommands(vec![command])
                    } else {
                        trace!(
                            "Failed to purchase transport for {:?}, this could be normal if we lack resources",
                            self.resource
                        );
                        GoalResult::TryAgainLater
                    }
                }
            },
            ResourceLinkState::TrainsPurchased { .. } => GoalResult::Finished,
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
