use std::collections::{HashMap, HashSet};

use bevy::app::{App, FixedUpdate};
use bevy::prelude::{
    debug, in_state, info, EventWriter, IntoSystemConfigs, Plugin, Res, ResMut, Resource, Time,
    Timer, TimerMode,
};
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::station_info::StationInfo;
use shared_domain::cargo_map::WithCargo;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::game_state::GameState;
use shared_domain::resource_type::ResourceType;
use shared_domain::transport::movement_orders::{MovementOrder, MovementOrders};
use shared_domain::transport::tile_track::TileTrack;
use shared_domain::transport::track_pathfinding::find_route_to_tile_tracks;
use shared_domain::transport::track_planner::plan_tracks;
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::transport::transport_type::TransportType;
use shared_domain::{IndustryBuildingId, PlayerId, StationId, TransportId};
use shared_util::random::choose;

use crate::communication::domain::ClientMessageEvent;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::states::ClientState;

#[derive(Resource)]
pub struct ArtificialIntelligenceTimer {
    timer: Option<Timer>,
}

impl ArtificialIntelligenceTimer {
    #[must_use]
    pub fn disabled() -> Self {
        Self { timer: None }
    }

    pub fn disable(&mut self) {
        info!("Disabling AI timer");
        self.timer = None;
    }

    pub fn enable(&mut self, seconds: f32) {
        info!("Enabling AI timer: {seconds} seconds");
        self.timer = Some(Timer::from_seconds(seconds, TimerMode::Repeating));
    }
}

pub struct ArtificialIntelligencePlugin;

impl Plugin for ArtificialIntelligencePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ArtificialIntelligenceTimer::disabled());
        app.add_systems(
            FixedUpdate,
            update_timer.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            act_upon_timer.run_if(in_state(ClientState::Playing)),
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
fn update_timer(time: Res<Time>, mut timer: ResMut<ArtificialIntelligenceTimer>) {
    if let Some(timer) = timer.timer.as_mut() {
        timer.tick(time.delta());
    }
}

#[allow(clippy::needless_pass_by_value)]
fn act_upon_timer(
    timer: Res<ArtificialIntelligenceTimer>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    player_id_resource: Res<PlayerIdResource>,
    game_state_resource: Res<GameStateResource>,
) {
    if let Some(ref timer) = timer.timer {
        if timer.just_finished() {
            let PlayerIdResource(player_id) = *player_id_resource;
            let GameStateResource(game_state) = game_state_resource.as_ref();

            ai_step(player_id, game_state, &mut client_messages);
        }
    }
}

fn ai_step(
    player_id: PlayerId,
    game_state: &GameState,
    client_messages: &mut EventWriter<ClientMessageEvent>,
) {
    let commands = try_building_transports(player_id, game_state)
        .or_else(|| try_building_tracks(player_id, game_state))
        .or_else(|| try_building_stations(player_id, game_state))
        .or_else(|| try_building_industry_buildings(player_id, game_state));

    if let Some(commands) = commands {
        for command in commands {
            info!("AI chose command: {:?}", command);
            client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                game_state.game_id(),
                command,
            )));
        }
    } else {
        info!("AI has nothing to do");
    }
}

#[allow(clippy::redundant_else)]
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
                .filter(|zoning| zoning.zoning_type() == industry_type.required_zoning())
                .collect();
            if let Some(chosen) = candidates.first() {
                let info = IndustryBuildingInfo::new(
                    player_id,
                    IndustryBuildingId::random(),
                    chosen.reference_tile(),
                    industry_type,
                );
                return Some(vec![GameCommand::BuildIndustryBuilding(info)]);
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
            .filter(|station_info| game_state.can_build_station(player_id, station_info))
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

#[allow(clippy::redundant_else)]
fn try_building_tracks(player_id: PlayerId, game_state: &GameState) -> Option<Vec<GameCommand>> {
    let connections = track_connections(game_state, logistics_links(player_id, game_state));
    // Later: Should we do this in random order?
    for (source, targets) in connections {
        if find_route_to_tile_tracks(source, &targets, game_state.building_state()).is_none() {
            // Later: Is picking just one of the targets the right thing to do?
            let target = targets.first()?;
            if let Some(route) = plan_tracks(
                player_id,
                EdgeXZ::from_tile_and_direction(source.tile_coords_xz, source.pointing_in),
                EdgeXZ::from_tile_and_direction(target.tile_coords_xz, target.pointing_in),
                game_state,
            ) {
                if !route.is_empty() {
                    // If it's empty, it means it's already built
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
        let matching_resource = transport
            .cargo_capacity()
            .resource_types_present()
            .contains(&resource_type);
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
    let transport_location = from_station
        .transport_location_at_station(tile_track.tile_coords_xz, tile_track.pointing_in)?;

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
