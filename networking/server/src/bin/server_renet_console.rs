use bevy::log::LogPlugin;
use bevy::prelude::{info, App};
use bevy::MinimalPlugins;
use networking_renet_server::server::networking::MultiplayerRenetServerPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(LogPlugin::default());
    app.add_plugins(MultiplayerRenetServerPlugin);
    info!("Starting server...");
    app.run();
}
