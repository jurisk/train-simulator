use bevy::app::App;
use bevy::prelude::{EventReader, Plugin, Update};
use bevy::utils::info;
use shared_domain::server_response::{LobbyResponse, ServerResponse};

use crate::communication::domain::ServerMessageEvent;

pub(crate) struct LobbyHandlerPlugin;

impl Plugin for LobbyHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_lobby_responses);
    }
}

// Later: Implement
fn handle_lobby_responses(mut server_messages: EventReader<ServerMessageEvent>) {
    for message in server_messages.read() {
        if let ServerResponse::Lobby(lobby_response) = &message.response {
            match lobby_response {
                LobbyResponse::AvailableGames(_game_infos) => {
                    info(format!("{lobby_response:?}"));
                },
                LobbyResponse::GameJoined(_game_info) => {
                    info(format!("{lobby_response:?}"));
                },
                LobbyResponse::GameLeft(_game_id) => {
                    info(format!("{lobby_response:?}"));
                },
            }
        }
    }
}
