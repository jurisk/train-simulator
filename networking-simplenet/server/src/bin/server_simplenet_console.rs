use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::app::App;
use bevy::log::LogPlugin;
use bevy::MinimalPlugins;
use networking_simplenet_server::MultiplayerSimpleNetServerPlugin;
use networking_simplenet_shared::DEFAULT_PORT;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let address = match args.get(1).cloned() {
        None => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), DEFAULT_PORT),
        Some(address_string) => {
            address_string
                .parse()
                .unwrap_or_else(|_| panic!("Unable to parse socket address {address_string}"))
        },
    };

    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(LogPlugin::default());
    app.add_plugins(MultiplayerSimpleNetServerPlugin { address });

    app.run();
}
