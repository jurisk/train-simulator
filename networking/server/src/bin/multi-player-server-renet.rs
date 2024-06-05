#![allow(clippy::unwrap_used)]

use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::prelude::{error, info, App, EventReader, Res, ResMut, Update};
use bevy::DefaultPlugins;
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_renet::renet::transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::renet::{ConnectionConfig, RenetServer, ServerEvent};
use bevy_renet::transport::NetcodeServerPlugin;
use bevy_renet::{renet, RenetServerPlugin};
use game_logic::server_state::ServerState;
use networking_renet_shared::channels::{ClientChannel, ServerChannel};
use networking_renet_shared::ServerStateResource;
use renet_visualizer::RenetServerVisualizer;
use shared_domain::client_command::{ClientCommand, ClientCommandWithClientId};
use shared_domain::server_response::ServerResponseWithClientIds;
use shared_domain::ClientId;

// Later: Could clean it up to merge with client code and avoid unwraps, but it doesn't matter
fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);

    app.insert_resource(ServerStateResource(ServerState::new()));

    app.add_plugins(RenetServerPlugin);

    let server = RenetServer::new(ConnectionConfig::default());
    app.insert_resource(server);

    app.add_plugins(NetcodeServerPlugin);
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let server_config = ServerConfig {
        current_time:     SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        max_clients:      64,
        protocol_id:      0,
        public_addresses: vec![server_addr],
        authentication:   ServerAuthentication::Unsecure,
    };
    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    app.insert_resource(transport);

    app.add_systems(Update, receive_message_system);
    app.add_systems(Update, handle_events_system);

    app.add_plugins(EguiPlugin);

    app.insert_resource(RenetServerVisualizer::<200>::default());
    app.add_systems(Update, update_visualizer_system);

    app.run();
}

#[allow(clippy::needless_pass_by_value)]
fn update_visualizer_system(
    mut egui_contexts: EguiContexts,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    server: Res<RenetServer>,
) {
    visualizer.update(&server);
    visualizer.show_window(egui_contexts.ctx_mut());
}

fn receive_message_system(
    mut server: ResMut<RenetServer>,
    mut server_state_resource: ResMut<ServerStateResource>,
) {
    let ServerStateResource(ref mut server_state) = server_state_resource.as_mut();

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Command) {
            match bincode::deserialize::<ClientCommand>(&message) {
                Ok(command) => {
                    info!("Received {command:?}");
                    let responses = server_state.process(ClientCommandWithClientId {
                        client_id: ClientId::from_raw(client_id.raw()),
                        command,
                    });
                    process_responses(&mut server, responses);
                },
                Err(error) => {
                    error!("Failed to deserialize {message:?}: {error}");
                },
            }
        }
    }
}

fn process_responses(
    server: &mut ResMut<RenetServer>,
    responses: Vec<ServerResponseWithClientIds>,
) {
    for response in responses {
        info!("Sending {response:?}...");
        for client_id in response.client_ids {
            match bincode::serialize(&response.response) {
                Ok(encoded) => {
                    server.send_message(
                        renet::ClientId::from_raw(client_id.raw()),
                        ServerChannel::ServerMessages,
                        encoded,
                    );
                },
                Err(error) => {
                    error!("Failed to deserialize {:?}: {error}", response.response);
                },
            }
        }
    }
}

fn handle_events_system(
    mut server_events: EventReader<ServerEvent>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Client {client_id} connected");
                visualizer.add_client(*client_id);
            },
            ServerEvent::ClientDisconnected { client_id, reason } => {
                visualizer.remove_client(*client_id);
                info!("Client {client_id} disconnected: {reason}");
            },
        }
    }
}
