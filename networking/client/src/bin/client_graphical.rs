use bevy::prelude::App;
use client_graphics::states::ClientState;
use client_graphics::ClientGraphicsPlugin;
use networking_client::MultiplayerSimpleNetClientPlugin;
use networking_shared::WEBSOCKETS_PORT;

#[cfg(target_arch = "wasm32")]
fn ws_url_from_window_location() -> Option<String> {
    let window = web_sys::window().expect("no global `window` exists");
    let location = window.location();
    let origin = location.origin().expect("should have origin");
    let origin_url = web_sys::Url::new(&origin).ok()?;

    let ws_url = origin_url.clone();
    if origin_url.protocol().starts_with(&"https") {
        // It can be "https:"...
        ws_url.set_protocol("wss");
        // If it's `https` then we assume ingress is doing SSL termination and it stays on the same port
    } else {
        ws_url.set_protocol("ws");
        ws_url.set_port(&format!("{WEBSOCKETS_PORT}"));
    }
    ws_url.set_pathname("/ws");

    Some(ws_url.to_string().into())
}

#[cfg(not(target_arch = "wasm32"))]
fn ws_url_from_window_location() -> Option<String> {
    None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let url = match args.get(1).cloned() {
        None => {
            let url_string = ws_url_from_window_location()
                .unwrap_or_else(|| format!("ws://127.0.0.1:{WEBSOCKETS_PORT}/ws"));
            url::Url::parse(url_string.as_str())?
        },
        Some(address_string) => {
            address_string
                .parse()
                .unwrap_or_else(|_| panic!("Unable to parse URL {address_string}"))
        },
    };

    let mut app = App::new();

    app.add_plugins(ClientGraphicsPlugin);
    app.insert_state(ClientState::ConnectingToServer);

    app.add_plugins(MultiplayerSimpleNetClientPlugin { url });

    app.run();

    Ok(())
}
