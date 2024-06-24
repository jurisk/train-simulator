#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;

use bevy::prelude::{
    in_state, info, Commands, EventReader, EventWriter, FixedUpdate, IntoSystemConfigs, NextState,
    OnEnter, Plugin, Res, ResMut, Resource,
};
use shared_domain::client_command::GameCommand::{QueryBuildings, QueryTransports};
use shared_domain::client_command::{
    AccessToken, AuthenticationCommand, ClientCommand, LobbyCommand,
};
use shared_domain::game_state::GameState;
use shared_domain::server_response::{
    AuthenticationResponse, GameResponse, PlayerInfo, ServerResponse,
};
use shared_domain::{GameId, PlayerId};

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::buildings::BuildingsPlugin;
use crate::game::map_level::MapLevelPlugin;
use crate::game::transport::TransportPlugin;
use crate::states::ClientState;

mod buildings;
pub mod map_level;
mod transport;

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
        app.add_systems(FixedUpdate, handle_players_updated);
        app.add_systems(
            FixedUpdate,
            handle_login_successful.run_if(in_state(ClientState::LoggingIn)),
        );
        app.insert_resource(PlayersInfoResource(HashMap::default()));
        app.add_systems(FixedUpdate, handle_game_state_snapshot);
    }
}

#[derive(Resource)]
pub struct PlayerIdResource(pub PlayerId);

// TODO HIGH: Replace with `GameStateResource` sub-component
#[derive(Resource)]
pub struct PlayersInfoResource(pub HashMap<PlayerId, PlayerInfo>);

#[derive(Resource)]
pub struct GameIdResource(pub GameId);

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
    mut players_info: ResMut<PlayersInfoResource>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, GameResponse::PlayersUpdated(new_player_infos)) =
            &message.response
        {
            let PlayersInfoResource(player_infos) = players_info.as_mut();
            player_infos.clone_from(new_player_infos);
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
                commands.insert_resource(GameIdResource(*game_id));
                client_state.set(ClientState::Playing);

                // We do it like this, because we need the `MapLevelResource` to be set before we can render buildings, so we don't want to receive them too early
                client_messages.send(ClientMessageEvent {
                    command: ClientCommand::Game(*game_id, QueryBuildings),
                });

                client_messages.send(ClientMessageEvent {
                    command: ClientCommand::Game(*game_id, QueryTransports),
                });
            }
        }
    }
}
