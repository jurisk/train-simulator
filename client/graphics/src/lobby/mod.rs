use bevy::prelude::{
    in_state, warn, App, EventReader, EventWriter, FixedUpdate, IntoSystemConfigs, Plugin, Res,
};
use shared_domain::client_command::{ClientCommand, LobbyCommand};
use shared_domain::server_response::{GameInfo, LobbyResponse, ServerResponse};
use shared_domain::{GameId, UserId};

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::{GameLaunchParams, UserIdResource};
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
    user_id: UserId,
) -> Option<GameId> {
    let game_with_user = games
        .iter()
        .find(|game_info| {
            game_info
                .user_players
                .iter()
                .any(|(that_user_id, _player_id)| *that_user_id == user_id)
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

    if game_with_user.is_some()
        && game_matching_game_id.is_some()
        && game_with_user != game_matching_game_id
    {
        warn!(
            "Player is in a game {game_with_user:?} that doesn't match the game ID provided {game_matching_game_id:?}"
        );
    }

    game_matching_game_id
        .or(game_with_user)
        .or(game_matching_map_id)
}

#[expect(clippy::needless_pass_by_value)]
fn handle_available_games(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    game_launch_params: Res<GameLaunchParams>,
    user_id_resource: Res<UserIdResource>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Lobby(LobbyResponse::AvailableGames(games)) = &message.response {
            let UserIdResource(user_id) = user_id_resource.as_ref();
            let game_launch_params = game_launch_params.as_ref();

            let selected = select_game_to_join(games, game_launch_params, *user_id);

            let command = match selected {
                None => {
                    LobbyCommand::CreateGame(game_launch_params.map_id.clone().unwrap_or_default())
                },
                Some(game_id) => LobbyCommand::JoinExistingGame(game_id),
            };

            client_messages.send(ClientMessageEvent::new(ClientCommand::Lobby(command)));
        }
    }
}
