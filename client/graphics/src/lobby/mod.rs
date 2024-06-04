use bevy::app::App;
use bevy::prelude::{EventReader, EventWriter, Plugin, Update};
use shared_domain::client_command::{ClientCommand, LobbyCommand};
use shared_domain::server_response::{LobbyResponse, ServerResponse};
use shared_domain::PlayerName;

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};

pub(crate) struct LobbyHandlerPlugin;

impl Plugin for LobbyHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_available_games);
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
