use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::app::App;
use bevy::log::LogPlugin;
use bevy::MinimalPlugins;
use networking_simplenet_server::MultiplayerSimpleNetServerPlugin;
use networking_simplenet_shared::DEFAULT_PORT;

fn main() {
    // TODO: Read from args
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), DEFAULT_PORT);

    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(LogPlugin::default());
    app.add_plugins(MultiplayerSimpleNetServerPlugin { address });

    app.run();
}
