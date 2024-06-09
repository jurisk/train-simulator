use std::error::Error;

use bevy::prelude::App;
use bevy::DefaultPlugins;
use networking_renet_server::server::networking::MultiplayerRenetServerPlugin;
use networking_renet_server::server::networking_visualisation::MultiplayerRenetServerVisualisationPlugin;
use networking_renet_shared::parse_address;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let address_string: Option<String> = args.get(1).cloned();
    let address = parse_address(address_string)?;

    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(MultiplayerRenetServerPlugin::new(address));
    app.add_plugins(MultiplayerRenetServerVisualisationPlugin);
    app.run();

    Ok(())
}
