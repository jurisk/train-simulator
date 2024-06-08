use bevy::app::Update;
use bevy::prelude::{EventReader, EventWriter, OnEnter, Plugin, Res, Resource};
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
pub mod map_level;

#[allow(clippy::module_name_repetitions)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(BuildingsPlugin);
        app.add_plugins(MapLevelPlugin);
        app.add_systems(OnEnter(ClientState::JoiningGame), initiate_login);
        app.add_systems(Update, handle_login_successful);
        app.insert_resource(PlayerIdResource(PlayerId::random()));
    }
}

#[derive(Resource)]
pub struct PlayerIdResource(pub PlayerId);

#[allow(clippy::needless_pass_by_value)]
fn initiate_login(
    mut client_messages: EventWriter<ClientMessageEvent>,
    player_id_resource: Res<PlayerIdResource>,
) {
    let PlayerIdResource(player_id) = *player_id_resource;
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
