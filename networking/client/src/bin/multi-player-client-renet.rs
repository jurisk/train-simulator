#![allow(clippy::needless_pass_by_value, clippy::unwrap_used)]

use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::log::info;
use bevy::prelude::{
    error, App, ButtonInput, EventReader, EventWriter, IntoSystemConfigs, KeyCode, Local,
    NextState, Res, ResMut, Update,
};
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_renet::client_just_connected;
use bevy_renet::renet::transport::{ClientAuthentication, NetcodeClientTransport};
use bevy_renet::renet::{ConnectionConfig, RenetClient};
use bevy_renet::transport::NetcodeClientPlugin;
use bevy_renet::RenetClientPlugin;
use client_graphics::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use client_graphics::states::ClientState;
use client_graphics::ClientGraphicsPlugin;
use networking_renet_shared::channels::{ClientChannel, ServerChannel};
use renet_visualizer::{RenetClientVisualizer, RenetVisualizerStyle};
use shared_domain::server_response::ServerResponse;

// Later: Could clean it up to merge with server code and avoid unwraps, but it doesn't matter
fn main() {
    let mut app = App::new();

    app.insert_state(ClientState::ConnectingToServer);

    app.add_plugins(ClientGraphicsPlugin);

    app.add_plugins(RenetClientPlugin);

    let client = RenetClient::new(ConnectionConfig::default());
    app.insert_resource(client);

    app.add_plugins(NetcodeClientPlugin);

    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let client_id = u64::try_from(current_time.as_millis()).unwrap();

    let authentication = ClientAuthentication::Unsecure {
        server_addr,
        client_id,
        user_data: None,
        protocol_id: 0,
    };
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    app.insert_resource(transport);

    app.add_systems(Update, receive_message_system);

    app.add_systems(Update, client_send_player_commands);

    app.add_systems(
        Update,
        switch_from_connecting_to_connected_state.run_if(client_just_connected),
    );

    if !app.is_plugin_added::<EguiPlugin>() {
        app.add_plugins(EguiPlugin);
    }
    app.insert_resource(RenetClientVisualizer::<200>::new(
        RenetVisualizerStyle::default(),
    ));
    app.add_systems(Update, update_visualizer_system);

    app.run();
}

fn update_visualizer_system(
    mut egui_contexts: EguiContexts,
    mut visualizer: ResMut<RenetClientVisualizer<200>>,
    client: Res<RenetClient>,
    mut show_visualizer: Local<bool>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    visualizer.add_network_info(client.network_info());
    if keyboard_input.just_pressed(KeyCode::F1) {
        *show_visualizer = !*show_visualizer;
    }
    if *show_visualizer {
        visualizer.show_window(egui_contexts.ctx_mut());
    }
}

fn switch_from_connecting_to_connected_state(mut client_state: ResMut<NextState<ClientState>>) {
    info!("Switching from ConnectingToServer to JoiningGame...");
    client_state.set(ClientState::JoiningGame);
}

fn receive_message_system(
    mut client: ResMut<RenetClient>,
    mut server_messages: EventWriter<ServerMessageEvent>,
) {
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        match bincode::deserialize::<ServerResponse>(&message) {
            Ok(response) => {
                info!("Sending server message: {response:?}");
                server_messages.send(ServerMessageEvent::new(response));
            },
            Err(error) => {
                error!("Failed to deserialize message {message:?}: {error:?}");
            },
        }
    }
}

pub fn client_send_player_commands(
    mut player_commands: EventReader<ClientMessageEvent>,
    mut client: ResMut<RenetClient>,
) {
    for command in player_commands.read() {
        let command_message = bincode::serialize(&command.command);
        match command_message {
            Ok(command_message) => {
                info!("Sending command: {command:?}");
                client.send_message(ClientChannel::Command, command_message);
            },
            Err(error) => error!("Failed to serialize {command:?}: {error:?}"),
        }
    }
}
