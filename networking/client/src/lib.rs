use bevy::prelude::{
    default, error, info, trace, warn, App, EventReader, EventWriter, FixedUpdate, NextState,
    Plugin, ResMut,
};
use bevy_simplenet::{
    AuthRequest, Client, ClientConfig, ClientEventFrom, ClientFactory, ClientReport,
};
use client_graphics::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use client_graphics::states::ClientState;
use enfync::builtin::Handle;
use networking_shared::{EncodedClientMsg, EncodedServerMsg, GameChannel};
use shared_domain::server_response::{GameResponse, ServerResponse};
use url::Url;

pub type GameClientEvent = ClientEventFrom<GameChannel>;

pub struct MultiplayerSimpleNetClientPlugin {
    pub url: Url,
}

impl Plugin for MultiplayerSimpleNetClientPlugin {
    fn build(&self, app: &mut App) {
        // Note that examples use SystemTime::now(), but it fails on WASM: https://github.com/rust-lang/rust/issues/48564
        let client_id = fastrand::u128(..);
        let client = client_factory().new_client(
            Handle::default(), // automatically selects native/WASM runtime
            self.url.clone(),
            AuthRequest::None { client_id },
            ClientConfig {
                reconnect_on_disconnect: true,
                reconnect_on_server_close: true,
                ..default()
            },
            (),
        );

        app.insert_resource(client);

        app.add_systems(FixedUpdate, read_on_client);
        app.add_systems(FixedUpdate, client_send_player_commands);
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn client_send_player_commands(
    mut player_commands: EventReader<ClientMessageEvent>,
    client: ResMut<Client<GameChannel>>,
) {
    for command in player_commands.read() {
        let command_message = bincode::serialize(&command.command);
        match command_message {
            Ok(command_message) => {
                info!("Sending command: {command:?}");
                client.send(EncodedClientMsg(command_message));
            },
            Err(error) => error!("Failed to serialize {command:?}: {error:?}"),
        }
    }
}

fn read_on_client(
    mut client: ResMut<Client<GameChannel>>,
    mut client_state: ResMut<NextState<ClientState>>,
    mut server_messages: EventWriter<ServerMessageEvent>,
) {
    while let Some(client_event) = client.next() {
        match client_event {
            GameClientEvent::Report(connection_report) => {
                info!("Connection report: {connection_report:?}");

                match connection_report {
                    ClientReport::Connected => client_state.set(ClientState::LoggingIn),
                    ClientReport::ClosedByServer(_)
                    | ClientReport::ClosedBySelf
                    | ClientReport::Disconnected => {
                        client_state.set(ClientState::ConnectingToServer);
                    },
                    ClientReport::IsDead(_pending_requests) => panic!("Client is dead"),
                }
            },
            GameClientEvent::Msg(EncodedServerMsg(message)) => {
                match bincode::deserialize::<ServerResponse>(&message) {
                    Ok(response) => {
                        if matches!(
                            response,
                            ServerResponse::Game(_, GameResponse::DynamicInfosSync(_, _, _, _))
                        ) {
                            trace!("Received server message: {response:?}");
                        } else {
                            info!("Received server message: {response:?}");
                        }

                        server_messages.send(ServerMessageEvent::new(response));
                    },
                    Err(error) => {
                        error!("Failed to deserialize message {message:?}: {error:?}");
                    },
                }
            },
            GameClientEvent::Response(response, request_id) => {
                warn!("Unexpected response: {response:?} {request_id}");
            },
            GameClientEvent::Ack(request_id) => trace!("Ack: {request_id}"),
            GameClientEvent::Reject(request_id) => trace!("Reject: {request_id}"),
            GameClientEvent::SendFailed(request_id) => warn!("Send failed: {request_id}"),
            GameClientEvent::ResponseLost(request_id) => warn!("Response lost: {request_id}"),
        }
    }
}

fn client_factory() -> ClientFactory<GameChannel> {
    ClientFactory::<GameChannel>::new(env!("CARGO_PKG_VERSION"))
}
