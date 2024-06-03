use std::collections::HashMap;

use shared_domain::game_state::GameState;
use shared_domain::server_response::GameInfo;
use shared_domain::GameId;

#[must_use]
#[allow(clippy::implicit_hasher)]
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
