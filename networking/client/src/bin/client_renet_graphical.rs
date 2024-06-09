use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::App;
use client_graphics::states::ClientState;
use client_graphics::ClientGraphicsPlugin;
use networking_renet_client::client::networking::MultiplayerRenetClientPlugin;
use networking_renet_client::client::networking_visualisation::MultiplayerRenetClientVisualisationPlugin;
use networking_renet_shared::DEFAULT_PORT;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let address_string: Option<String> = args.get(1).cloned();
    let address = match address_string {
        None => SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), DEFAULT_PORT),
        Some(address_string) => address_string.parse()?,
    };

    let mut app = App::new();

    app.add_plugins(ClientGraphicsPlugin);
    app.insert_state(ClientState::ConnectingToServer);
    app.add_plugins(MultiplayerRenetClientPlugin::new(address));
    app.add_plugins(MultiplayerRenetClientVisualisationPlugin);

    app.run();

    Ok(())
}
