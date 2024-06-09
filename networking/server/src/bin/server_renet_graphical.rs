use bevy::prelude::App;
use bevy::DefaultPlugins;
use networking_renet_server::server::networking::MultiplayerRenetServerPlugin;
use networking_renet_server::server::networking_visualisation::MultiplayerRenetServerVisualisationPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(MultiplayerRenetServerPlugin);
    app.add_plugins(MultiplayerRenetServerVisualisationPlugin);
    app.run();
}
