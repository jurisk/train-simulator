#![allow(clippy::module_name_repetitions)]

use bevy::asset::{Assets, Handle};
use bevy::color::Color;
use bevy::core::Name;
use bevy::log::error;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{
    default, in_state, info, Bundle, Commands, EventReader, EventWriter, FixedUpdate,
    IntoSystemConfigs, Mesh, NextState, OnEnter, Plugin, Res, ResMut, Resource, Time, Transform,
    Update,
};
use shared_domain::building::building_info::WithTileCoverage;
use shared_domain::client_command::GameCommand::{QueryBuildings, QueryTracks, QueryTransports};
use shared_domain::client_command::{
    AccessToken, AuthenticationCommand, ClientCommand, LobbyCommand,
};
use shared_domain::game_state::GameState;
use shared_domain::game_time::GameTimeDiff;
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::players::player_state::PlayerState;
use shared_domain::server_response::{
    AuthenticationResponse, Colour, GameResponse, ServerResponse,
};
use shared_domain::tile_coverage::TileCoverage;
use shared_domain::{GameId, PlayerId};

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::buildings::BuildingsPlugin;
use crate::game::map_level::MapLevelPlugin;
use crate::game::test_objects::build_test_objects;
use crate::game::transport::TransportPlugin;
use crate::states::ClientState;

pub mod buildings;
pub mod map_level;
mod test_objects;
pub mod transport;

#[derive(Resource)]
pub struct GameStateResource(pub GameState);

#[allow(clippy::module_name_repetitions)]
pub struct GamePlugin {
    pub game_launch_params: GameLaunchParams,
}

#[derive(Resource, Clone)]
pub struct GameLaunchParams {
    pub player_id:    PlayerId,
    pub access_token: AccessToken,
    pub game_id:      Option<GameId>,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(self.game_launch_params.clone());
        app.add_plugins(BuildingsPlugin);
        app.add_plugins(TransportPlugin);
        app.add_plugins(MapLevelPlugin);
        app.add_systems(OnEnter(ClientState::LoggingIn), initiate_login);
        app.add_systems(
            FixedUpdate,
            handle_players_updated.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            handle_login_successful.run_if(in_state(ClientState::LoggingIn)),
        );
        app.add_systems(FixedUpdate, handle_game_state_snapshot);
        app.add_systems(
            Update,
            client_side_time_advance.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            build_test_objects.run_if(in_state(ClientState::Playing)),
        );
    }
}

#[derive(Resource)]
pub struct PlayerIdResource(pub PlayerId);

// Movement prediction on the client side
#[allow(clippy::needless_pass_by_value)]
fn client_side_time_advance(mut game_state_resource: ResMut<GameStateResource>, time: Res<Time>) {
    let GameStateResource(ref mut game_state) = game_state_resource.as_mut();
    game_state.advance_time_diff(GameTimeDiff::from_seconds(time.delta_seconds()));
}

#[allow(clippy::needless_pass_by_value)]
fn initiate_login(
    mut client_messages: EventWriter<ClientMessageEvent>,
    game_launch_params: Res<GameLaunchParams>,
) {
    client_messages.send(ClientMessageEvent::new(ClientCommand::Authentication(
        AuthenticationCommand::Login(
            game_launch_params.player_id,
            game_launch_params.access_token.clone(),
        ),
    )));
}

fn handle_login_successful(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    mut commands: Commands,
) {
    for message in server_messages.read() {
        if let ServerResponse::Authentication(AuthenticationResponse::LoginSucceeded(player_id)) =
            &message.response
        {
            info!("Login successful, player_id: {player_id:?}");
            commands.insert_resource(PlayerIdResource(*player_id));

            client_messages.send(ClientMessageEvent::new(ClientCommand::Lobby(
                LobbyCommand::ListGames,
            )));
        }
    }
}

fn handle_players_updated(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut game_state_resource: ResMut<GameStateResource>,
) {
    let GameStateResource(ref mut game_state) = game_state_resource.as_mut();
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, GameResponse::PlayersUpdated(new_player_infos)) =
            &message.response
        {
            game_state.update_player_infos(new_player_infos);
        }
    }
}

// TODO: How does `terrain` differ from `map_level`? What about trees? Is it `MapLevel`? Is it `Buildings`?
#[allow(clippy::collapsible_match)]
fn handle_game_state_snapshot(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    mut client_state: ResMut<NextState<ClientState>>,
    mut commands: Commands,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(game_id, game_response) = &message.response {
            if let GameResponse::GameStateSnapshot(game_state) = game_response {
                commands.insert_resource(GameStateResource(game_state.clone()));
                client_state.set(ClientState::Playing);

                // Later:   We do it like this, because we need the `GameStateSnapshot` to be set
                //          before we can render buildings, so we don't want to receive them too
                //          early. Also, this way we actually create the client-side graphical
                //          objects using the same update mechanism, instead of immediately, now.
                //          This should be improved as we basically send buildings & transports
                //          twice now.
                for query in [QueryBuildings, QueryTracks, QueryTransports] {
                    client_messages.send(ClientMessageEvent {
                        command: ClientCommand::Game(*game_id, query),
                    });
                }
            }
        }
    }
}

#[allow(clippy::similar_names)]
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
#[allow(clippy::missing_panics_doc)]
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

#[allow(clippy::too_many_arguments, clippy::similar_names)]
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
