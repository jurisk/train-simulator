use std::collections::{BTreeSet, HashMap, HashSet};

use log::debug;
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::station_info::StationInfo;
use shared_domain::cargo_map::WithCargo;
use shared_domain::client_command::GameCommand;
use shared_domain::directional_edge::DirectionalEdge;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::resource_type::ResourceType;
use shared_domain::transport::movement_orders::{MovementOrder, MovementOrders};
use shared_domain::transport::tile_track::TileTrack;
use shared_domain::transport::track_planner::{plan_tracks, DEFAULT_ALREADY_EXISTS_COEF};
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::transport::transport_type::TransportType;
use shared_domain::{IndustryBuildingId, PlayerId, StationId, TransportId};
use shared_util::random::choose;

#[derive(Default)]
pub struct ArtificialIntelligenceState {
    track_connections_built: HashSet<BTreeSet<TileTrack>>,
}

pub fn ai_commands(
    player_id: PlayerId,
    game_state: &GameState,
    ai_state: &mut ArtificialIntelligenceState,
    metrics: &impl Metrics,
) -> Option<Vec<GameCommand>> {
    try_building_transports(player_id, game_state)
        .or_else(|| try_building_tracks(ai_state, player_id, game_state, metrics))
        .or_else(|| try_building_stations(player_id, game_state))
        .or_else(|| try_building_industry_buildings(player_id, game_state))
}

#[expect(clippy::redundant_else)]
fn try_building_industry_buildings(
    player_id: PlayerId,
    game_state: &GameState,
) -> Option<Vec<GameCommand>> {
    let free = game_state.all_free_zonings();

    for industry_type in IndustryType::all() {
        let existing = game_state
            .building_state()
            .find_industry_building_by_owner_and_type(player_id, industry_type);
        if existing.is_empty() {
            let candidates: Vec<_> = free
                .iter()
                .filter(|zoning| Some(zoning.zoning_type()) == industry_type.required_zoning())
                .map(|zoning| {
                    IndustryBuildingInfo::new(
                        player_id,
                        IndustryBuildingId::random(),
                        zoning.reference_tile(),
                        industry_type,
                    )
                })
                .filter(|info| {
                    game_state
                        .can_build_industry_building(player_id, info)
                        .is_ok()
                })
                .collect();

            // TODO: If industry has no zoning requirement, build in an empty space, but choose the best place.
            if let Some(info) = candidates.first() {
                return Some(vec![GameCommand::BuildIndustryBuilding(info.clone())]);
            } else {
                debug!("No free zoning for {:?}", industry_type);
            }
        }
    }

    None
}

fn try_building_stations(player_id: PlayerId, game_state: &GameState) -> Option<Vec<GameCommand>> {
    for industry_building in game_state
        .building_state()
        .find_players_industry_buildings_without_linked_stations(player_id)
    {
        let options = industry_building
            .candidate_station_locations()
            .into_iter()
            .map(|(tile, station_type)| {
                StationInfo::new(player_id, StationId::random(), tile, station_type)
            })
            .filter(|station_info| {
                game_state
                    .can_build_station(player_id, station_info)
                    .is_ok()
            })
            .collect::<Vec<_>>();

        // Later: Don't choose randomly, but the "best" (not sure what that means yet) location
        match choose(&options) {
            None => {
                debug!("No station locations for {:?}", industry_building);
            },
            Some(selected) => {
                return Some(vec![GameCommand::BuildStation(selected.clone())]);
            },
        }
    }

    None
}

fn logistics_links(
    player_id: PlayerId,
    game_state: &GameState,
) -> Vec<(StationId, ResourceType, StationId)> {
    let mut results = vec![];
    let buildings = game_state.building_state();
    let stations = buildings.find_players_stations(player_id);
    for from_station in &stations {
        for to_station in &stations {
            if from_station.id() != to_station.id() {
                let from_resources = from_station.cargo().resource_types_present();
                let to_resources = buildings.resource_types_accepted_by_station(to_station.id());
                for resource_type in from_resources.intersection(&to_resources) {
                    results.push((from_station.id(), *resource_type, to_station.id()));
                }
            }
        }
    }
    results
}

// Later: Should the connections be `DirectionalEdge`-s instead of `TileTrack`?
fn track_connections(
    game_state: &GameState,
    links: Vec<(StationId, ResourceType, StationId)>,
) -> HashMap<TileTrack, Vec<TileTrack>> {
    let unique_station_pairs = links
        .into_iter()
        // If we don't do bidirectional links then we never bring the empty train back to the source
        .flat_map(|(from, _, to)| vec![(from, to), (to, from)])
        .collect::<HashSet<_>>();
    let mut results = HashMap::new();
    for (from_station_id, to_station_id) in unique_station_pairs {
        let from_station = game_state.building_state().find_station(from_station_id);
        let to_station = game_state.building_state().find_station(to_station_id);
        let from_tracks = from_station
            .map(StationInfo::station_exit_tile_tracks)
            .unwrap_or_default();
        let to_tracks = to_station
            .map(StationInfo::station_exit_tile_tracks)
            .unwrap_or_default();
        for from_track in &from_tracks {
            results.insert(*from_track, to_tracks.clone());
        }
    }
    results
}

fn try_building_tracks(
    ai_state: &mut ArtificialIntelligenceState,
    player_id: PlayerId,
    game_state: &GameState,
    metrics: &impl Metrics,
) -> Option<Vec<GameCommand>> {
    let connections = track_connections(game_state, logistics_links(player_id, game_state));
    for (source, targets) in connections {
        for target in targets {
            let edge_set = BTreeSet::from_iter([source, target]);
            if ai_state.track_connections_built.contains(&edge_set) {
                // We have built this before...
                continue;
            }
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
                    continue;
                }
                if game_state.can_build_tracks(player_id, &route).is_ok() {
                    ai_state.track_connections_built.insert(edge_set);
                    return Some(vec![GameCommand::BuildTracks(route)]);
                }
            } else {
                debug!("No route found for {:?} -> {:?}", source, target);
            }
        }
    }

    None
}

fn matching_transport_exists(
    transports: &[&TransportInfo],
    from_station: StationId,
    resource_type: ResourceType,
    to_station: StationId,
) -> bool {
    transports.iter().any(|transport| {
        let matching_resource = transport.cargo_capacity().contains_resource(resource_type);
        let orders = transport.movement_orders();
        let matching_from = orders.contains_station(from_station);
        let matching_to = orders.contains_station(to_station);
        matching_resource && matching_from && matching_to
    })
}

fn purchase_transport_command(
    player_id: PlayerId,
    game_state: &GameState,
    from_station: StationId,
    resource_type: ResourceType,
    to_station: StationId,
) -> Option<GameCommand> {
    let mut movement_orders = MovementOrders::one(MovementOrder::stop_at_station(from_station));
    movement_orders.push(MovementOrder::stop_at_station(to_station));

    let from_station = game_state.building_state().find_station(from_station)?;
    let tile_tracks = from_station.station_exit_tile_tracks();
    let tile_track = tile_tracks.first()?;
    let transport_location =
        from_station.transport_location_at_station(tile_track.tile, tile_track.pointing_in)?;

    let command = GameCommand::PurchaseTransport(TransportInfo::new(
        TransportId::random(),
        player_id,
        TransportType::cargo_train(resource_type),
        transport_location,
        movement_orders,
    ));

    Some(command)
}

fn try_building_transports(
    player_id: PlayerId,
    game_state: &GameState,
) -> Option<Vec<GameCommand>> {
    let transports = game_state
        .transport_state()
        .find_players_transports(player_id);

    for (from_station, resource_type, to_station) in logistics_links(player_id, game_state) {
        if !matching_transport_exists(&transports, from_station, resource_type, to_station) {
            return purchase_transport_command(
                player_id,
                game_state,
                from_station,
                resource_type,
                to_station,
            )
            .map(|command| vec![command]);
        }
    }

    None
}
