#![allow(clippy::module_name_repetitions)]

use std::str::FromStr;

use bevy::asset::{Assets, Handle};
use bevy::color::Color;
use bevy::core::Name;
use bevy::log::{error, warn};
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{
    Bundle, Commands, EventReader, EventWriter, FixedUpdate, IntoSystemConfigs, Mesh, NextState,
    OnEnter, Plugin, Res, ResMut, Resource, Time, Transform, Update, default, in_state, info,
    trace,
};
use shared_domain::building::building_info::WithTileCoverage;
use shared_domain::client_command::{
    AccessToken, AuthenticationCommand, ClientCommand, LobbyCommand,
};
use shared_domain::game_state::GameState;
use shared_domain::game_time::GameTimeDiff;
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::metrics::NoopMetrics;
use shared_domain::players::player_state::PlayerState;
use shared_domain::server_response::{
    AuthenticationResponse, Colour, GameResponse, ServerResponse,
};
use shared_domain::tile_coverage::TileCoverage;
use shared_domain::{GameId, PlayerId, ScenarioId, UserId};
use shared_util::tap::TapErr;

use crate::ai::ArtificialIntelligencePlugin;
use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::buildings::BuildingsPlugin;
use crate::game::map_level::MapLevelPlugin;
use crate::game::transport::TransportPlugin;
use crate::states::ClientState;

pub mod buildings;
pub mod map_level;
pub mod transport;

#[derive(Resource)]
pub struct GameStateResource(pub GameState);

#[expect(clippy::module_name_repetitions)]
pub struct GamePlugin {
    pub game_launch_params: GameLaunchParams,
}

// Later: Improve to make invalid combinations impossible on type level
#[derive(Resource, Clone, Debug)]
pub struct GameLaunchParams {
    pub user_id:      UserId,
    pub access_token: AccessToken,
    pub game_id:      Option<GameId>,
    pub scenario_id:  Option<ScenarioId>,
}

impl GameLaunchParams {
    #[must_use]
    pub fn new(user_id: &str, access_token: &str, scenario_id: &str, game_id: &str) -> Self {
        let user_id = UserId::from_str(user_id).unwrap_or_else(|err| {
            warn!("Invalid user ID {user_id:?}: {err}");
            UserId::random()
        });
        let access_token = AccessToken::new(access_token.to_string());
        let scenario_id = ScenarioId::from_str(scenario_id)
            .tap_err(|err| warn!("Invalid scenario ID {scenario_id:?}: {err:?}"))
            .ok();
        let game_id = GameId::from_str(game_id)
            .tap_err(|err| warn!("Invalid game ID {game_id:?}: {err:?}"))
            .ok();

        Self {
            user_id,
            access_token,
            game_id,
            scenario_id,
        }
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(self.game_launch_params.clone());
        app.add_plugins(BuildingsPlugin);
        app.add_plugins(TransportPlugin);
        app.add_plugins(MapLevelPlugin);
        app.add_plugins(ArtificialIntelligencePlugin);
        app.add_systems(OnEnter(ClientState::LoggingIn), initiate_login);
        app.add_systems(
            FixedUpdate,
            handle_players_updated.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            handle_login_successful.run_if(in_state(ClientState::LoggingIn)),
        );
        app.add_systems(FixedUpdate, handle_game_joining_and_game_state_snapshot);
        app.add_systems(
            Update,
            client_side_time_advance.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(FixedUpdate, handle_errors);
    }
}

#[derive(Resource)]
pub struct UserIdResource(pub UserId);

#[derive(Resource)]
pub struct PlayerIdResource(pub PlayerId);

// Movement prediction on the client side
#[expect(clippy::needless_pass_by_value)]
fn client_side_time_advance(mut game_state_resource: ResMut<GameStateResource>, time: Res<Time>) {
    let GameStateResource(ref mut game_state) = game_state_resource.as_mut();
    game_state.advance_time_diff(
        GameTimeDiff::from_seconds(time.delta_seconds()),
        &NoopMetrics::default(),
    );
}

#[expect(clippy::needless_pass_by_value)]
fn initiate_login(
    mut client_messages: EventWriter<ClientMessageEvent>,
    game_launch_params: Res<GameLaunchParams>,
) {
    client_messages.send(ClientMessageEvent::new(ClientCommand::Authentication(
        AuthenticationCommand::Login(
            game_launch_params.user_id,
            game_launch_params.access_token.clone(),
        ),
    )));
}

fn handle_login_successful(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    mut commands: Commands,
    mut client_state: ResMut<NextState<ClientState>>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Authentication(AuthenticationResponse::LoginSucceeded(user_id)) =
            &message.response
        {
            info!("Login successful, user_id: {user_id:?}");
            commands.insert_resource(UserIdResource(*user_id));

            client_state.set(ClientState::JoiningGame);

            client_messages.send(ClientMessageEvent::new(ClientCommand::Lobby(
                LobbyCommand::ListGames,
            )));
        }
    }
}

fn handle_players_updated(mut server_messages: EventReader<ServerMessageEvent>) {
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, GameResponse::PlayersUpdated(new_player_infos)) =
            &message.response
        {
            trace!("Players updated: {new_player_infos:?}");
            // Later: Consider if we use this somewhere
        }
    }
}

#[expect(clippy::match_same_arms)]
fn handle_game_joining_and_game_state_snapshot(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_state: ResMut<NextState<ClientState>>,
    mut commands: Commands,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            match game_response {
                GameResponse::GameStateSnapshot(snapshot) => {
                    commands.insert_resource(GameStateResource(snapshot.clone()));
                },
                GameResponse::PlayersUpdated(_) => {},
                GameResponse::IndustryBuildingAdded(_) => {},
                GameResponse::IndustryBuildingRemoved(_) => {},
                GameResponse::StationAdded(_) => {},
                GameResponse::StationRemoved(_) => {},
                GameResponse::TracksAdded(_) => {},
                GameResponse::TracksRemoved(_) => {},
                GameResponse::TransportsAdded(_) => {},
                GameResponse::DynamicInfosSync(..) => {},
                GameResponse::GameJoined(player_id, snapshot) => {
                    commands.insert_resource(GameStateResource(snapshot.clone()));
                    commands.insert_resource(PlayerIdResource(*player_id));
                    client_state.set(ClientState::Playing);
                },
                GameResponse::GameLeft => {},
                GameResponse::Error(_) => {},
            }
        }
    }
}

fn handle_errors(mut server_messages: EventReader<ServerMessageEvent>) {
    for message in server_messages.read() {
        match &message.response {
            ServerResponse::Authentication(AuthenticationResponse::Error(error)) => {
                error!("Authentication error: {error:?}");
            },
            ServerResponse::Game(_, GameResponse::Error(error)) => {
                error!("Game error: {error:?}");
            },
            ServerResponse::Error(error) => {
                error!("Server error: {error:?}");
            },
            _ => {},
        }
    }
}

#[expect(clippy::similar_names)]
pub(crate) fn player_colour(players_info: &PlayerState, player_id: PlayerId) -> Colour {
    match players_info.get(player_id) {
        None => {
            error!(
                "Player with ID {:?} not found, returning invalid colour",
                player_id
            );
            Colour::rgb(255, 0, 0)
        },
        Some(player_info) => player_info.colour,
    }
}

#[must_use]
pub fn center_vec3(object: &dyn WithTileCoverage, map_level: &MapLevel) -> Vec3 {
    let terrain = map_level.terrain();
    let (nw, se) = match object.covers_tiles() {
        TileCoverage::Single(tile) => (tile, tile),
        TileCoverage::Rectangular {
            north_west_inclusive,
            south_east_inclusive,
        } => (north_west_inclusive, south_east_inclusive),
    };
    let nw = nw.vertex_coords_nw();
    let se = se.vertex_coords_se();
    let nw = terrain.logical_to_world(nw);
    let se = terrain.logical_to_world(se);

    (se + nw) / 2.0
}

#[expect(clippy::too_many_arguments, clippy::similar_names)]
pub fn create_object_entity(
    object: &dyn WithTileCoverage,
    label: String,
    colour: Colour,
    mesh: Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    commands: &mut Commands,
    map_level: &MapLevel,
    additional: impl Bundle,
) {
    let center = center_vec3(object, map_level);
    let color = Color::srgb_u8(colour.r, colour.g, colour.b);
    let material = materials.add(color);

    // TODO: Make buildings distinguishable from each other - e.g. use `label` to also draw text on the sides / roof of the building

    let mut commands = commands.spawn((
        PbrBundle {
            transform: Transform {
                translation: center,
                ..default()
            },
            material,
            mesh,
            ..default()
        },
        Name::new(label),
    ));
    commands.insert(additional);
}
