#![allow(clippy::implicit_hasher, clippy::missing_errors_doc)]

use std::collections::HashMap;

use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::server_response::{
    AddressEnvelope, GameInfo, GameResponse, LobbyResponse, ServerError, ServerResponse,
    ServerResponseWithAddress,
};
use shared_domain::{GameId, PlayerId, PlayerName};

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

pub fn join_game(
    game_state: &mut GameState,
    game_id: GameId,
    requesting_player_id: PlayerId,
    requesting_player_name: PlayerName,
) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
    // Later: Don't allow joining multiple games

    game_state
        .players
        .insert(requesting_player_id, requesting_player_name);

    Ok(vec![
        ServerResponseWithAddress::new(
            AddressEnvelope::ToAllPlayersInGame(game_id),
            ServerResponse::Lobby(LobbyResponse::GameJoined(game_id)),
        ),
        ServerResponseWithAddress::new(
            AddressEnvelope::ToPlayer(requesting_player_id),
            ServerResponse::Game(GameResponse::State(game_state.clone())),
        ),
    ])
}

pub fn create_and_join_game(
    games: &mut HashMap<GameId, GameState>,
    game_prototype: &GameState,
    requesting_player_id: PlayerId,
    requesting_player_name: PlayerName,
) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
    // Later: Don't allow starting a game if is already a part of another game?

    let game_id = GameId::random();

    let mut game_state = game_prototype.clone();

    let results = join_game(
        &mut game_state,
        game_id,
        requesting_player_id,
        requesting_player_name,
    )?;

    games.insert(game_id, game_state);

    Ok(results)
}
