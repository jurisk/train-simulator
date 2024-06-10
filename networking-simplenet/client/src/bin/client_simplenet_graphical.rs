use bevy::prelude::App;
use client_graphics::states::ClientState;
use client_graphics::ClientGraphicsPlugin;
use networking_simplenet_client::MultiplayerSimpleNetClientPlugin;
use networking_simplenet_shared::DEFAULT_PORT;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = url::Url::parse(format!("ws://127.0.0.1:{DEFAULT_PORT}/ws").as_str())?;

    let mut app = App::new();

    app.add_plugins(ClientGraphicsPlugin);
    app.insert_state(ClientState::ConnectingToServer);

    app.add_plugins(MultiplayerSimpleNetClientPlugin { url });

    app.run();

    Ok(())
}
