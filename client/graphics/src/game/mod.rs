#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;

use bevy::prelude::{
    EventReader, EventWriter, FixedUpdate, OnEnter, Plugin, Res, ResMut, Resource,
};
use shared_domain::client_command::{
    AccessToken, AuthenticationCommand, ClientCommand, LobbyCommand,
};
use shared_domain::server_response::{
    AuthenticationResponse, GameResponse, PlayerInfo, ServerResponse,
};
use shared_domain::{GameId, PlayerId};

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::buildings::BuildingsPlugin;
use crate::game::map_level::MapLevelPlugin;
use crate::game::vehicles::VehiclesPlugin;
use crate::states::ClientState;

mod buildings;
pub mod map_level;
mod vehicles;

#[allow(clippy::module_name_repetitions)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(BuildingsPlugin);
        app.add_plugins(VehiclesPlugin);
        app.add_plugins(MapLevelPlugin);
        app.add_systems(OnEnter(ClientState::JoiningGame), initiate_login);
        app.add_systems(FixedUpdate, handle_players_updated);
        app.add_systems(FixedUpdate, handle_login_successful);
        app.insert_resource(PlayerIdResource(PlayerId::random())); // TODO: Improve auto-login so it only kicks in when an environment variable is set
        app.insert_resource(GameIdResource(GameId::random())); // Questionable, but dealing with it being missing may be worse
        app.insert_resource(PlayersInfoResource(HashMap::default()));
    }
}

#[derive(Resource)]
pub struct PlayerIdResource(pub PlayerId);

#[derive(Resource)]
pub struct PlayersInfoResource(pub HashMap<PlayerId, PlayerInfo>);

#[derive(Resource)]
pub struct GameIdResource(pub GameId);

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
            client_messages.send(ClientMessageEvent::new(ClientCommand::Lobby(
                LobbyCommand::ListGames,
            )));
        }
    }
}

fn handle_players_updated(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut players_info: ResMut<PlayersInfoResource>,
    mut game_id_resource: ResMut<GameIdResource>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(game_id, GameResponse::PlayersUpdated(new_player_infos)) =
            &message.response
        {
            let PlayersInfoResource(player_infos) = players_info.as_mut();
            player_infos.clone_from(new_player_infos);

            // Questionable we do it every time the players change, but this will have to do for now
            let GameIdResource(game_id_resource) = game_id_resource.as_mut();
            game_id_resource.clone_from(game_id);
        }
    }
}
