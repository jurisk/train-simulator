use bevy::prelude::{warn, App, EventReader, EventWriter, FixedUpdate, Plugin, Res};
use shared_domain::client_command::{ClientCommand, LobbyCommand};
use shared_domain::server_response::{LobbyResponse, ServerResponse};

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::{GameLaunchParams, PlayerIdResource};

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
    player_id_resource: Option<Res<PlayerIdResource>>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Lobby(LobbyResponse::AvailableGames(games)) = &message.response {
            let player_id = player_id_resource.as_ref().map(|res| res.0);

            let game_with_player = games
                .iter()
                .find(|game_info| {
                    game_info
                        .players
                        .iter()
                        .any(|player_info| Some(player_info.id) == player_id)
                })
                .map(|game_info| game_info.game_id);

            let game_matching_game_id = game_launch_params
                .game_id
                .filter(|game_id| games.iter().any(|game_info| game_info.game_id == *game_id));

            let map_id = game_launch_params.map_id.clone().unwrap_or_default();
            let game_matching_map_id = games
                .iter()
                .find(|game_info| game_info.map_id == map_id)
                .map(|game_info| game_info.game_id);
            let first_game = games.first().map(|game_info| game_info.game_id);

            if game_with_player.is_some()
                && game_matching_game_id.is_some()
                && game_with_player != game_matching_game_id
            {
                warn!(
                    "Player is in a game {game_with_player:?} that doesn't match the game ID provided {game_matching_game_id:?}"
                );
            }

            let selected_game_id = game_matching_game_id
                .or(game_with_player)
                .or(game_matching_map_id)
                .or(first_game);

            let command = match selected_game_id {
                None => LobbyCommand::CreateGame(map_id),
                Some(game_id) => LobbyCommand::JoinExistingGame(game_id),
            };

            client_messages.send(ClientMessageEvent::new(ClientCommand::Lobby(command)));
        }
    }
}
