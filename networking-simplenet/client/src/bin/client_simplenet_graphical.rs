use bevy::prelude::App;
use client_graphics::states::ClientState;
use client_graphics::ClientGraphicsPlugin;
use networking_simplenet_client::MultiplayerSimpleNetClientPlugin;
use networking_simplenet_shared::WEBSOCKETS_PORT;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let url = match args.get(1).cloned() {
        None => url::Url::parse(format!("ws://127.0.0.1:{WEBSOCKETS_PORT}/ws").as_str())?,
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
