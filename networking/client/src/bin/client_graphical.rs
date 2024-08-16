use bevy::prelude::AppExtStates;
use bevy::prelude::{info, App};
use clap::Parser;
use client_graphics::game::GameLaunchParams;
use client_graphics::states::ClientState;
use client_graphics::ClientGraphicsPlugin;
use fastrand as _;
use networking_client::MultiplayerSimpleNetClientPlugin;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    url:       Option<String>,
    #[clap(short, long)]
    player_id: Option<String>,
    #[clap(short, long)]
    map_id:    Option<String>,
    #[clap(short, long)]
    game_id:   Option<String>,
}

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
fn main() {
    let args = Args::parse();
    let url = args
        .url
        .unwrap_or_else(|| format!("ws://127.0.0.1:{}/ws", networking_shared::PORT));

    run_with_string(
        url.as_str(),
        &args.player_id.unwrap_or_default(),
        &args.map_id.unwrap_or_default(),
        &args.game_id.unwrap_or_default(),
    );
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
