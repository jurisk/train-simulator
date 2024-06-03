#![allow(clippy::implicit_hasher, clippy::missing_errors_doc)]

use std::collections::HashMap;

use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::server_response::{
    AddressEnvelope, GameInfo, GameResponse, ServerError, ServerResponse, ServerResponseWithAddress,
};
use shared_domain::{GameId, PlayerId};

#[must_use]
pub fn create_game_infos(games: &HashMap<GameId, GameState>) -> Vec<GameInfo> {
    games
        .iter()
        .map(|(game_id, game_state)| {
            GameInfo {
                game_id: *game_id,
                players: game_state.players.clone(),
            }
        })
        .collect()
}

pub fn process_game_command(
    game_id: GameId,
    game_state: &mut GameState,
    _requesting_player_id: PlayerId,
    game_command: GameCommand,
) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
    match game_command {
        GameCommand::BuildBuilding(building_info) => {
            // TODO: Check that `requesting_player_id` can build there, assign ownership

            game_state.buildings.push(building_info.clone());

            Ok(vec![ServerResponseWithAddress::new(
                AddressEnvelope::ToAllPlayersInGame(game_id),
                ServerResponse::Game(GameResponse::BuildingBuilt(building_info.clone())),
            )])
        },
    }
}

pub fn lookup_game_state(
    games: &mut HashMap<GameId, GameState>,
    game_id: GameId,
) -> Result<&mut GameState, ServerResponse> {
    match games.get_mut(&game_id) {
        None => Err(ServerResponse::Error(ServerError::GameNotFound)),
        Some(result) => Ok(result),
    }
}
