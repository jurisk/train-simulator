use bevy::app::{App, FixedUpdate};
use bevy::prelude::{
    debug, in_state, info, EventWriter, IntoSystemConfigs, Plugin, Res, ResMut, Resource, Time,
    Timer, TimerMode,
};
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::station_info::StationInfo;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::game_state::GameState;
use shared_domain::resource_type::ResourceType;
use shared_domain::transport::tile_track::TileTrack;
use shared_domain::transport::track_planner::plan_tracks;
use shared_domain::{IndustryBuildingId, PlayerId, StationId};
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
            debug!("AI chose command: {:?}", command);
            client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                game_state.game_id(),
                command,
            )));
        }
    } else {
        debug!("AI has nothing to do");
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
    // TODO HIGH: Implement
    results
}

fn track_connections(
    game_state: &GameState,
    links: Vec<(StationId, ResourceType, StationId)>,
) -> Vec<(TileTrack, TileTrack)> {
    let mut results = vec![];
    for (from_station_id, _, to_station_id) in links {
        let from_station = game_state.building_state().find_station(from_station_id);
        let to_station = game_state.building_state().find_station(to_station_id);
        let from_tracks = from_station
            .map(StationInfo::station_exit_tile_tracks)
            .unwrap_or_default();
        let to_tracks = to_station
            .map(StationInfo::station_exit_tile_tracks)
            .unwrap_or_default();
        for from_track in &from_tracks {
            for to_track in &to_tracks {
                results.push((*from_track, *to_track));
            }
        }
    }
    results
}

#[allow(clippy::redundant_else)]
fn try_building_tracks(player_id: PlayerId, game_state: &GameState) -> Option<Vec<GameCommand>> {
    let connections = track_connections(game_state, logistics_links(player_id, game_state));
    // TODO HIGH: Don't do choose unsafe, do safe choose
    let (a, b) = choose(&connections)?;
    // TODO HIGH: If route already exists (empty results from `plan_tracks`?!), no need to build it again, try the next one!
    if let Some(route) = plan_tracks(
        player_id,
        &[],
        &[
            EdgeXZ::from_tile_and_direction(a.tile_coords_xz, a.pointing_in),
            EdgeXZ::from_tile_and_direction(b.tile_coords_xz, b.pointing_in),
        ],
        game_state.building_state(),
        game_state.map_level(),
    ) {
        return Some(vec![GameCommand::BuildTracks(route)]);
    } else {
        debug!("No route found for {:?} -> {:?}", a, b);
    }
    None
}

fn try_building_transports(
    _player_id: PlayerId,
    _game_state: &GameState,
) -> Option<Vec<GameCommand>> {
    // TODO HIGH: Implement
    None
}
