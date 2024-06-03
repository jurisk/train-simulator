use bevy::app::App;
use bevy::prelude::{Commands, EventReader, EventWriter, NextState, Plugin, ResMut, Update};
use shared_domain::client_command::{ClientCommand, LobbyCommand};
use shared_domain::server_response::{GameResponse, LobbyResponse, ServerResponse};
use shared_domain::PlayerName;

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::GameStateResource;
use crate::states::ClientState;

pub(crate) struct LobbyHandlerPlugin;

impl Plugin for LobbyHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_available_games);
        app.add_systems(Update, handle_game_joined);
    }
}

fn handle_available_games(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_messages: EventWriter<ClientMessageEvent>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Lobby(LobbyResponse::AvailableGames(games)) = &message.response {
            let player_name = PlayerName::random();
            let command = match games.first() {
                None => LobbyCommand::CreateGame(player_name),
                Some(game_info) => LobbyCommand::JoinExistingGame(game_info.game_id, player_name),
            };

            client_messages.send(ClientMessageEvent::new(ClientCommand::Lobby(command)));
        }
    }
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
