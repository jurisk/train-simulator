use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::SystemTime;

use bevy::app::Update;
use bevy::log::{error, info};
use bevy::prelude::{App, EventReader, EventWriter, IntoSystemConfigs, NextState, Plugin, ResMut};
use bevy_renet::renet::transport::{ClientAuthentication, NetcodeClientTransport};
use bevy_renet::renet::{ConnectionConfig, RenetClient};
use bevy_renet::transport::NetcodeClientPlugin;
use bevy_renet::{client_just_connected, RenetClientPlugin};
use client_graphics::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use client_graphics::states::ClientState;
use networking_renet_shared::channels::{ClientChannel, ServerChannel};
use shared_domain::server_response::ServerResponse;

pub struct MultiplayerRenetClientPlugin {
    server_address: SocketAddr,
}

impl MultiplayerRenetClientPlugin {
    #[must_use]
    pub fn new(server_address: SocketAddr) -> Self {
        Self { server_address }
    }
}

// Note: We were also considering https://github.com/ukoehb/bevy_simplenet
impl Plugin for MultiplayerRenetClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetClientPlugin);

        let client = RenetClient::new(ConnectionConfig::default());
        app.insert_resource(client);

        app.add_plugins(NetcodeClientPlugin);

        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Failed to get current time");

        let client_id =
            u64::try_from(current_time.as_millis()).expect("Failed to create client ID");

        let server_addr = self.server_address;

        let authentication = ClientAuthentication::Unsecure {
            server_addr,
            client_id,
            user_data: None,
            protocol_id: 0,
        };

        let self_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0);
        let socket = UdpSocket::bind(self_socket)
            .unwrap_or_else(|_| panic!("Failed to bind to {self_socket}"));
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Failed to get current time");
        let transport = NetcodeClientTransport::new(current_time, authentication, socket)
            .expect("Failed to create client transport");

        app.insert_resource(transport);

        app.add_systems(Update, receive_message_system);

        app.add_systems(Update, client_send_player_commands);

        app.add_systems(
            Update,
            switch_from_connecting_to_connected_state.run_if(client_just_connected),
        );
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
