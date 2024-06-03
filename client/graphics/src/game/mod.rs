use bevy::app::Update;
use bevy::prelude::{
    in_state, Commands, EventReader, EventWriter, IntoSystemConfigs, NextState, OnEnter, Plugin,
    ResMut, Resource,
};
use shared_domain::client_command::{
    AccessToken, AuthenticationCommand, ClientCommand, LobbyCommand,
};
use shared_domain::game_state::GameState;
use shared_domain::server_response::{GameResponse, ServerResponse};
use shared_domain::PlayerId;

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::buildings::BuildingsPlugin;
use crate::game::map_level::MapLevelPlugin;
use crate::states::ClientState;

mod buildings;
mod map_level;

#[allow(clippy::module_name_repetitions)]
pub struct GamePlugin;

#[allow(clippy::module_name_repetitions)]
#[derive(Resource)]
pub struct GameStateResource {
    game_state: GameState,
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(BuildingsPlugin);
        app.add_plugins(MapLevelPlugin);
        app.add_systems(OnEnter(ClientState::Joining), game_startup);
        app.add_systems(
            Update,
            handle_game_joined.run_if(in_state(ClientState::Joining)),
        );
    }
}

fn game_startup(mut client_messages: EventWriter<ClientMessageEvent>) {
    let player_id = PlayerId::random();
    client_messages.send(ClientMessageEvent::new(ClientCommand::Authentication(
        AuthenticationCommand::Login(player_id, AccessToken("valid-token".to_string())),
    )));

    client_messages.send(ClientMessageEvent::new(ClientCommand::Lobby(
        LobbyCommand::CreateGame,
    )));
}

#[allow(clippy::collapsible_match)]
fn handle_game_joined(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_state: ResMut<NextState<ClientState>>,
    mut commands: Commands,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(game_response) = &message.response {
            if let GameResponse::State(game_state) = game_response {
                commands.insert_resource(GameStateResource {
                    game_state: game_state.clone(),
                });
                client_state.set(ClientState::Playing);
            }
        }
    }
}
