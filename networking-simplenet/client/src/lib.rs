use std::time::{SystemTime, UNIX_EPOCH};

use bevy::app::{App, Update};
use bevy::prelude::{
    error, info, trace, warn, EventReader, EventWriter, NextState, Plugin, ResMut,
};
use bevy_simplenet::{
    AuthRequest, Client, ClientConfig, ClientEventFrom, ClientFactory, ClientReport,
};
use client_graphics::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use client_graphics::states::ClientState;
use networking_simplenet_shared::{TestChannel, TestClientMsg, TestConnectMsg, TestServerMsg};
use shared_domain::server_response::ServerResponse;
use url::Url;

pub type TestClientEvent = ClientEventFrom<TestChannel>;

pub struct MultiplayerSimpleNetClientPlugin {
    pub url: Url,
}

impl Plugin for MultiplayerSimpleNetClientPlugin {
    fn build(&self, app: &mut App) {
        let client_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let client = client_factory().new_client(
            enfync::builtin::Handle::default(), // automatically selects native/WASM runtime
            self.url.clone(),
            AuthRequest::None { client_id },
            ClientConfig {
                reconnect_on_disconnect: true,
                reconnect_on_server_close: true,
                ..Default::default()
            },
            TestConnectMsg(String::from("hello")),
        );

        app.insert_resource(client);

        app.add_systems(Update, read_on_client);
        app.add_systems(Update, client_send_player_commands);
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn client_send_player_commands(
    mut player_commands: EventReader<ClientMessageEvent>,
    client: ResMut<Client<TestChannel>>,
) {
    for command in player_commands.read() {
        let command_message = bincode::serialize(&command.command);
        match command_message {
            Ok(command_message) => {
                info!("Sending command: {command:?}");
                client.send(TestClientMsg(command_message));
            },
            Err(error) => error!("Failed to serialize {command:?}: {error:?}"),
        }
    }
}

fn read_on_client(
    mut client: ResMut<Client<TestChannel>>,
    mut client_state: ResMut<NextState<ClientState>>,
    mut server_messages: EventWriter<ServerMessageEvent>,
) {
    while let Some(client_event) = client.next() {
        match client_event {
            TestClientEvent::Report(connection_report) => {
                info!("Connection report: {connection_report:?}");

                match connection_report {
                    ClientReport::Connected => client_state.set(ClientState::JoiningGame),
                    ClientReport::ClosedByServer(_)
                    | ClientReport::ClosedBySelf
                    | ClientReport::Disconnected => {
                        client_state.set(ClientState::ConnectingToServer);
                    },
                    ClientReport::IsDead(_pending_requests) => panic!("Client is dead"),
                }
            },
            TestClientEvent::Msg(TestServerMsg(message)) => {
                match bincode::deserialize::<ServerResponse>(&message) {
                    Ok(response) => {
                        info!("Received server message: {response:?}");
                        server_messages.send(ServerMessageEvent::new(response));
                    },
                    Err(error) => {
                        error!("Failed to deserialize message {message:?}: {error:?}");
                    },
                }
            },
            TestClientEvent::Response(response, request_id) => {
                warn!("Unexpected response: {response:?} {request_id}");
            },
            TestClientEvent::Ack(request_id) => trace!("Ack: {request_id}"),
            TestClientEvent::Reject(request_id) => trace!("Reject: {request_id}"),
            TestClientEvent::SendFailed(request_id) => warn!("Send failed: {request_id}"),
            TestClientEvent::ResponseLost(request_id) => warn!("Response lost: {request_id}"),
        }
    }
}

fn client_factory() -> ClientFactory<TestChannel> {
    // You must use the same protocol version string as the server factory.
    ClientFactory::<TestChannel>::new("test")
}
