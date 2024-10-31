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

use crate::oct2025::industries::IndustryState;
use crate::oct2025::stations::{lookup_station, lookup_station_id};
use crate::oct2025::transports::purchase_transport_command;

#[derive(Clone)]
pub(crate) enum ResourceLinkState {
    Pending,
    TracksPending(Vec<(TileTrack, TileTrack)>),
    TracksBuilt,
    PurchasingTrain(TransportId),
    TrainPurchased,
}

impl ResourceLinkState {
    #[expect(clippy::collapsible_else_if, clippy::too_many_lines)]
    #[must_use]
    pub(crate) fn commands(
        &mut self,
        from_industry_state: &IndustryState,
        resource: ResourceType,
        to_industry_state: &IndustryState,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>> {
        match self {
            ResourceLinkState::Pending => {
                let from_station = lookup_station(from_industry_state, game_state)?;
                let to_station = lookup_station(to_industry_state, game_state)?;

                let mut pairs = vec![];
                for track_a in from_station.station_exit_tile_tracks() {
                    for track_b in to_station.station_exit_tile_tracks() {
                        pairs.push((track_a, track_b));
                        pairs.push((track_b, track_a));
                    }
                }
                *self = ResourceLinkState::TracksPending(pairs);
                self.commands(
                    from_industry_state,
                    resource,
                    to_industry_state,
                    player_id,
                    game_state,
                    metrics,
                )
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
                            self.commands(
                                from_industry_state,
                                resource,
                                to_industry_state,
                                player_id,
                                game_state,
                                metrics,
                            )
                        } else {
                            if game_state.can_build_tracks(player_id, &route).is_ok() {
                                Some(vec![GameCommand::BuildTracks(route)])
                            } else {
                                Some(vec![])
                            }
                        }
                    } else {
                        debug!("No route found for {:?} -> {:?}", source, target);
                        self.commands(
                            from_industry_state,
                            resource,
                            to_industry_state,
                            player_id,
                            game_state,
                            metrics,
                        )
                    }
                } else {
                    *self = ResourceLinkState::TracksBuilt;
                    self.commands(
                        from_industry_state,
                        resource,
                        to_industry_state,
                        player_id,
                        game_state,
                        metrics,
                    )
                }
            },
            ResourceLinkState::TracksBuilt => {
                let from_station = lookup_station_id(from_industry_state)?;
                let to_station = lookup_station_id(to_industry_state)?;

                let command = purchase_transport_command(
                    player_id,
                    game_state,
                    from_station,
                    resource,
                    to_station,
                )?;

                match &command {
                    GameCommand::PurchaseTransport(_, transport) => {
                        *self = ResourceLinkState::PurchasingTrain(transport.transport_id());
                    },
                    _ => {
                        error!("Unexpected command for creating transport: {command:?}");
                    },
                }

                Some(vec![command])
            },
            ResourceLinkState::PurchasingTrain(transport_id) => {
                if game_state
                    .transport_state()
                    .find_players_transport(player_id, *transport_id)
                    .is_some()
                {
                    *self = ResourceLinkState::TrainPurchased;
                }

                None
            },
            ResourceLinkState::TrainPurchased => None,
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
