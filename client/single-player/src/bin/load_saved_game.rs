use std::env::args;

use client_graphics::game::GameLaunchParams;
use client_single_player::run;
use shared_domain::client_command::AccessToken;
use shared_domain::game_state::{GameState, GameStateFlattened};
use shared_domain::{PlayerId, UserId};
use shared_util::compression::load_from_bytes;

fn parse_player_id(game_state: &GameState, value: Option<String>) -> Option<PlayerId> {
    // TODO: This doesn't actually get the n-th player because `infos_cloned()` returns random results, we should instead get sorted PlayerId-s and pick from those
    value
        .and_then(|s| s.parse::<usize>().ok())
        .and_then(|idx| game_state.players().infos_cloned().get(idx).cloned())
        .map(|info| info.id)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Later: Could read the filename from args but for now we can just hardcode
    let input_path = "ai_until_final_goods_built.game_state.bincode.gz";
    let bytes = std::fs::read(input_path)?;
    let game_state: GameStateFlattened = load_from_bytes(&bytes)?;
    let game_state: GameState = game_state.into();
    let player_id = parse_player_id(&game_state, args().nth(1));

    let params = GameLaunchParams {
        user_id: UserId::random(),
        access_token: AccessToken::new("valid-token".to_string()),
        game_id: None,
        scenario_id: None,
        game_state: Some(game_state),
        player_id,
    };

    run(params);

    Ok(())
}
