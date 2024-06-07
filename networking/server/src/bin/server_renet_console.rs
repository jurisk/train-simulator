use bevy::log::{info, LogPlugin};
use bevy::prelude::App;
use bevy::MinimalPlugins;
use networking_renet_server::server::networking::MultiPlayerRenetNetworkingPlugin;

// TODO: Make it a) work b) work under Docker
fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(LogPlugin::default());
    app.add_plugins(MultiPlayerRenetNetworkingPlugin);
    info!("Starting server...");
    app.run();
}
