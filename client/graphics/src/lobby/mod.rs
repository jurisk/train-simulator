use bevy::prelude::{App, EventReader, EventWriter, FixedUpdate, Plugin, Res};
use shared_domain::client_command::{ClientCommand, LobbyCommand};
use shared_domain::server_response::{LobbyResponse, ServerResponse};

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::GameLaunchParams;

pub(crate) struct LobbyHandlerPlugin;

impl Plugin for LobbyHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_available_games);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_available_games(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    game_launch_params: Res<GameLaunchParams>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Lobby(LobbyResponse::AvailableGames(games)) = &message.response {
            let game_id = game_launch_params
                .game_id
                .filter(|game_id| games.iter().any(|game_info| game_info.game_id == *game_id));

            let command = match game_id {
                None => {
                    match games.first() {
                        None => LobbyCommand::CreateGame,
                        Some(game_info) => LobbyCommand::JoinExistingGame(game_info.game_id),
                    }
                },
                Some(game_id) => LobbyCommand::JoinExistingGame(game_id),
            };

            client_messages.send(ClientMessageEvent::new(ClientCommand::Lobby(command)));
        }
    }
}
