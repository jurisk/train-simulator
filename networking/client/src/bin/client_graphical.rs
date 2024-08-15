use std::str::FromStr;

use bevy::log::warn;
use bevy::prelude::AppExtStates;
use bevy::prelude::{info, App};
use client_graphics::game::GameLaunchParams;
use client_graphics::states::ClientState;
use client_graphics::ClientGraphicsPlugin;
use fastrand as _;
use networking_client::MultiplayerSimpleNetClientPlugin;
use shared_domain::client_command::AccessToken;
use shared_domain::{GameId, MapId, PlayerId};
use shared_util::tap::TapErr;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start(url: &str, player_id: &str, map_id: &str, game_id: &str) {
    run_with_string(url, player_id, map_id, game_id);
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Deliberately empty as we actually want `start` called with a parameter from WASM
    println!("WASM main() called");
}

#[allow(clippy::expect_used)]
#[cfg(not(target_arch = "wasm32"))]
// TODO HIGH: Use https://github.com/TeXitoi/structopt for game launch params for the command line
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let url: String = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| format!("ws://127.0.0.1:{}/ws", networking_shared::PORT));

    let player_id = match args.get(2).cloned() {
        None => PlayerId::random(),
        Some(player_id) => PlayerId::from_str(player_id.as_str()).expect("Failed to parse UUID"),
    };

    run_with_string(url.as_str(), player_id.to_string().as_str(), "", "");
}

fn run_with_string(url: &str, player_id: &str, map_id: &str, game_id: &str) {
    let parsed_url =
        url::Url::parse(url).unwrap_or_else(|err| panic!("Invalid URL {url:?}: {err}"));
    let parsed_player_id = PlayerId::from_str(player_id).unwrap_or_else(|err| {
        warn!("Invalid player ID {player_id:?}: {err}");
        PlayerId::random()
    });
    let parsed_map_id = MapId::from_str(map_id)
        .tap_err(|err| warn!("Invalid map ID {map_id:?}: {err:?}"))
        .ok();
    let parsed_game_id = GameId::from_str(game_id)
        .tap_err(|err| warn!("Invalid game ID {game_id:?}: {err:?}"))
        .ok();
    run_with_url(parsed_url, parsed_player_id, parsed_map_id, parsed_game_id);
}

fn run_with_url(
    url: url::Url,
    player_id: PlayerId,
    map_id: Option<MapId>,
    game_id: Option<GameId>,
) {
    info!("Starting client with URL: {url}");
    let mut app = App::new();

    app.add_plugins(ClientGraphicsPlugin {
        game_launch_params: GameLaunchParams {
            player_id,
            access_token: AccessToken::new("valid-token".to_string()),
            game_id,
            map_id,
        },
    });
    app.insert_state(ClientState::ConnectingToServer);

    app.add_plugins(MultiplayerSimpleNetClientPlugin { url });

    app.run();
}
