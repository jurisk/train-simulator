use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::prelude::{App, ResMut, Update};
use bevy::MinimalPlugins;
use bevy_renet::renet::transport::{ClientAuthentication, NetcodeClientTransport};
use bevy_renet::renet::{ConnectionConfig, DefaultChannel, RenetClient};
use bevy_renet::transport::NetcodeClientPlugin;
use bevy_renet::RenetClientPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RenetClientPlugin);

    let client = RenetClient::new(ConnectionConfig::default());
    app.insert_resource(client);

    app.add_plugins(NetcodeClientPlugin);

    let server_addr = "127.0.0.1:5000".parse().unwrap();

    let authentication = ClientAuthentication::Unsecure {
        server_addr,
        client_id: 0,
        user_data: None,
        protocol_id: 0,
    };
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    app.insert_resource(transport);

    app.add_systems(Update, send_message_system);
    app.add_systems(Update, receive_message_system);

    app.run();
}

fn send_message_system(mut client: ResMut<RenetClient>) {
    // Send a text message to the server
    client.send_message(DefaultChannel::ReliableOrdered, "client message");
}

fn receive_message_system(mut client: ResMut<RenetClient>) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        // TODO: Handle received message
        println!("{message:?}");
    }
}
