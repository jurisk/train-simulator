use bevy::log::LogPlugin;
use bevy::prelude::{info, App};
use bevy::MinimalPlugins;
use networking_renet_server::server::networking::MultiPlayerRenetNetworkingPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(LogPlugin::default());
    app.add_plugins(MultiPlayerRenetNetworkingPlugin);
    info!("Starting server...");
    app.run();
}
