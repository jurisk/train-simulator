use client_graphics::game::GameLaunchParams;
use client_single_player::run;
use shared_domain::UserId;
use shared_domain::client_command::AccessToken;
use shared_domain::game_state::{GameState, GameStateFlattened};
use shared_util::compression::load_from_bytes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Later: Could read the filename from args but for now we can just hardcode
    let input_path = "ai_until_final_goods_built.game_state.bincode.gz";
    let bytes = std::fs::read(input_path)?;
    let game_state: GameStateFlattened =
        load_from_bytes(&bytes)?;
    let game_state: GameState = game_state.into();

    let params = GameLaunchParams {
        user_id:      UserId::random(),
        access_token: AccessToken::new("valid-token".to_string()),
        game_id:      None,
        scenario_id:  None,
        game_state:   Some(game_state),
    };

    run(params);

    Ok(())
}
