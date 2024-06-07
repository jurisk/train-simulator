use bevy::prelude::App;
use bevy::DefaultPlugins;
use networking_renet_server::server::networking::MultiPlayerRenetNetworkingPlugin;
use networking_renet_server::server::networking_visualisation::MultiPlayerRenetVisualisationPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(MultiPlayerRenetNetworkingPlugin);
    app.add_plugins(MultiPlayerRenetVisualisationPlugin);
    app.run();
}
