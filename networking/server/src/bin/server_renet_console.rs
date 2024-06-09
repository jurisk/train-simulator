use std::error::Error;

use bevy::log::LogPlugin;
use bevy::prelude::App;
use bevy::MinimalPlugins;
use networking_renet_server::server::networking::MultiplayerRenetServerPlugin;
use networking_renet_server::server::server_address;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let address_string: Option<String> = args.get(1).cloned();
    let address = server_address(address_string)?;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(LogPlugin::default());
    app.add_plugins(MultiplayerRenetServerPlugin::new(address));
    app.run();

    Ok(())
}
