#![allow(clippy::implicit_hasher, clippy::missing_errors_doc)]

use std::collections::HashMap;

use shared_domain::client_command::LobbyCommand;
use shared_domain::game_state::GameState;
use shared_domain::server_response::{
    AddressEnvelope, LobbyResponse, ServerResponse, ServerResponseWithAddress,
};
use shared_domain::{GameId, PlayerId};

use crate::game_logic::{create_and_join_game, create_game_infos, join_game, lookup_game_state};

pub fn process_lobby_command(
    games: &mut HashMap<GameId, GameState>,
    requesting_player_id: PlayerId,
    lobby_command: LobbyCommand,

    // TODO: This is odd that we are passing this here.
    game_prototype: &GameState,
) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
    match lobby_command {
        LobbyCommand::ListGames => {
            Ok(vec![ServerResponseWithAddress::new(
                AddressEnvelope::ToPlayer(requesting_player_id),
                ServerResponse::Lobby(LobbyResponse::AvailableGames(create_game_infos(games))),
            )])
        },
        LobbyCommand::CreateGame(player_name) => {
            create_and_join_game(games, game_prototype, requesting_player_id, player_name)
        },
        LobbyCommand::JoinExistingGame(game_id, player_name) => {
            let game_state = lookup_game_state(games, game_id)?;
            join_game(game_state, game_id, requesting_player_id, player_name)
        },
        LobbyCommand::LeaveGame(game_id) => {
            // Later: Not sure how this should even work if the player has buildings and vehicles owned in the game?
            let game_state = lookup_game_state(games, game_id)?;
            game_state.players.remove(&requesting_player_id);
            Ok(vec![ServerResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(game_id),
                ServerResponse::Lobby(LobbyResponse::GameLeft(game_id)),
            )])
        },
    }
}
