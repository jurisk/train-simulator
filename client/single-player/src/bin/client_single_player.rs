use bevy::prelude::AppExtStates;
use bevy::prelude::{
    App, EventReader, EventWriter, FixedUpdate, Res, ResMut, Resource, Time, debug,
};
use clap::Parser;
use client_graphics::ClientGraphicsPlugin;
use client_graphics::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use client_graphics::game::GameLaunchParams;
use client_graphics::states::ClientState;
use client_single_player::run;
use game_logic::server_state::ServerState;
use shared_domain::ClientId;
use shared_domain::client_command::ClientCommandWithClientId;
use shared_domain::game_time::GameTime;
use shared_domain::metrics::NoopMetrics;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[expect(clippy::struct_field_names)]
struct Args {
    #[clap(short, long)]
    player_id:   Option<String>,
    #[clap(short, long)]
    scenario_id: Option<String>,
    #[clap(short, long)]
    game_id:     Option<String>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start(player_id: &str, scenario_id: &str, game_id: &str) {
    run_with_string(player_id, scenario_id, game_id);
}

fn run_with_string(player_id: &str, scenario_id: &str, game_id: &str) {
    let access_token = "valid-token";
    let game_launch_params = GameLaunchParams::new(player_id, access_token, scenario_id, game_id);

    run(game_launch_params);
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Deliberately empty as we actually want `start` called with a parameter from WASM
    println!("WASM main() called");
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args = Args::parse();
    run_with_string(
        &args.player_id.unwrap_or_default(),
        &args.scenario_id.unwrap_or_default(),
        &args.game_id.unwrap_or_default(),
    );
}
