use bevy::prelude::{info, App};
use client_graphics::states::ClientState;
use client_graphics::ClientGraphicsPlugin;
use networking_client::MultiplayerSimpleNetClientPlugin;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start(url: &str) {
    run_with_string(url);
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Deliberately empty as we actually want `start` called with a parameter from WASM
    println!("WASM main() called");
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let url: String = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| format!("ws://127.0.0.1:{}/ws", networking_shared::PORT));

    run_with_string(url.as_str());
}

fn run_with_string(url: &str) {
    let parsed = url::Url::parse(url).unwrap_or_else(|err| panic!("Invalid URL {url:?}: {err}"));
    run_with_url(parsed);
}

fn run_with_url(url: url::Url) {
    info!("Starting client with URL: {url}");
    let mut app = App::new();

    app.add_plugins(ClientGraphicsPlugin);
    app.insert_state(ClientState::ConnectingToServer);

    app.add_plugins(MultiplayerSimpleNetClientPlugin { url });

    app.run();
}
