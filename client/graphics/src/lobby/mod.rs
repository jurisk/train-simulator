use bevy::prelude::{
    in_state, warn, App, EventReader, EventWriter, FixedUpdate, IntoSystemConfigs, Plugin, Res,
};
use shared_domain::client_command::{ClientCommand, LobbyCommand};
use shared_domain::server_response::{GameInfo, LobbyResponse, ServerResponse};
use shared_domain::{GameId, PlayerId};

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::{GameLaunchParams, PlayerIdResource};
use crate::states::ClientState;

pub(crate) struct LobbyHandlerPlugin;

impl Plugin for LobbyHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            handle_available_games.run_if(in_state(ClientState::JoiningGame)),
        );
    }
}

fn select_game_to_join(
    games: &[GameInfo],
    game_launch_params: &GameLaunchParams,
    player_id: PlayerId,
) -> Option<GameId> {
    let game_with_player = games
        .iter()
        .find(|game_info| {
            game_info
                .players
                .iter()
                .any(|player_info| player_info.id == player_id)
        })
        .map(|game_info| game_info.game_id);

    let game_matching_game_id = game_launch_params
        .game_id
        .filter(|game_id| games.iter().any(|game_info| game_info.game_id == *game_id));

    let game_matching_map_id = match &game_launch_params.map_id {
        None => games.first().map(|game_info| game_info.game_id),
        Some(map_id) => {
            games
                .iter()
                .find(|game_info| game_info.map_id == *map_id)
                .map(|game_info| game_info.game_id)
        },
    };

    if game_with_player.is_some()
        && game_matching_game_id.is_some()
        && game_with_player != game_matching_game_id
    {
        warn!(
            "Player is in a game {game_with_player:?} that doesn't match the game ID provided {game_matching_game_id:?}"
        );
    }

    game_matching_game_id
        .or(game_with_player)
        .or(game_matching_map_id)
}

#[allow(clippy::needless_pass_by_value)]
fn handle_available_games(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    game_launch_params: Res<GameLaunchParams>,
    player_id_resource: Res<PlayerIdResource>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Lobby(LobbyResponse::AvailableGames(games)) = &message.response {
            let PlayerIdResource(player_id) = player_id_resource.as_ref();
            let game_launch_params = game_launch_params.as_ref();

            let selected_game_id = select_game_to_join(games, game_launch_params, *player_id);

            let command = match selected_game_id {
                None => {
                    LobbyCommand::CreateGame(game_launch_params.map_id.clone().unwrap_or_default())
                },
                Some(game_id) => LobbyCommand::JoinExistingGame(game_id),
            };

            client_messages.send(ClientMessageEvent::new(ClientCommand::Lobby(command)));
        }
    }
}
