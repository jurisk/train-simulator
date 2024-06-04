use bevy::app::Update;
use bevy::prelude::{EventReader, EventWriter, OnEnter, Plugin};
use shared_domain::client_command::{
    AccessToken, AuthenticationCommand, ClientCommand, LobbyCommand,
};
use shared_domain::server_response::{AuthenticationResponse, ServerResponse};
use shared_domain::PlayerId;

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::buildings::BuildingsPlugin;
use crate::game::map_level::MapLevelPlugin;
use crate::states::ClientState;

mod buildings;
mod map_level;

#[allow(clippy::module_name_repetitions)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(BuildingsPlugin);
        app.add_plugins(MapLevelPlugin);
        app.add_systems(OnEnter(ClientState::Joining), initiate_login);
        app.add_systems(Update, handle_login_successful);
    }
}

fn initiate_login(mut client_messages: EventWriter<ClientMessageEvent>) {
    let player_id = PlayerId::random();
    client_messages.send(ClientMessageEvent::new(ClientCommand::Authentication(
        AuthenticationCommand::Login(player_id, AccessToken("valid-token".to_string())),
    )));
}

fn handle_login_successful(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_messages: EventWriter<ClientMessageEvent>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Authentication(AuthenticationResponse::LoginSucceeded(_player_id)) =
            &message.response
        {
            // We could insert player_id into resources
            client_messages.send(ClientMessageEvent::new(ClientCommand::Lobby(
                LobbyCommand::ListGames,
            )));
        }
    }
}
