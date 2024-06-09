use bevy::prelude::App;
use client_graphics::states::ClientState;
use client_graphics::ClientGraphicsPlugin;
use networking_renet_client::client::networking::MultiplayerRenetClientPlugin;
use networking_renet_client::client::networking_visualisation::MultiplayerRenetClientVisualisationPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(ClientGraphicsPlugin);
    app.insert_state(ClientState::ConnectingToServer);
    app.add_plugins(MultiplayerRenetClientPlugin);
    app.add_plugins(MultiplayerRenetClientVisualisationPlugin);

    app.run();
}
