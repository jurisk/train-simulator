use std::str::FromStr;

use bevy::prelude::AppExtStates;
use bevy::prelude::{info, App};
use client_graphics::game::GameLaunchParams;
use client_graphics::states::ClientState;
use client_graphics::ClientGraphicsPlugin;
use fastrand as _;
use networking_client::MultiplayerSimpleNetClientPlugin;
use shared_domain::PlayerId;

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
    let access_token = "valid-token";
    let game_launch_params = GameLaunchParams::new(player_id, access_token, map_id, game_id);

    run_with_url(parsed_url, game_launch_params);
}

fn run_with_url(url: url::Url, game_launch_params: GameLaunchParams) {
    info!("Starting client: {url} {game_launch_params:?}");
    let mut app = App::new();

    app.add_plugins(ClientGraphicsPlugin { game_launch_params });
    app.insert_state(ClientState::ConnectingToServer);

    app.add_plugins(MultiplayerSimpleNetClientPlugin { url });

    app.run();
}
