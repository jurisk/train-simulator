use std::net::SocketAddr;
use std::time::Duration;

use bevy::app::{App, Update};
use bevy::log::info;
use bevy::prelude::{default, error, Plugin, ResMut, Resource};
use bevy_simplenet::{
    AcceptorConfig, Authenticator, RateLimitConfig, Server, ServerConfig, ServerEventFrom,
    ServerFactory, ServerReport,
};
use game_logic::server_state::ServerState;
use networking_shared::{EncodedClientMsg, EncodedServerMsg, GameChannel};
use shared_domain::client_command::{ClientCommand, ClientCommandWithClientId};
use shared_domain::server_response::ServerResponseWithClientIds;
use shared_domain::ClientId;

pub type GameServerEvent = ServerEventFrom<GameChannel>;

#[derive(Resource)]
struct ServerStateResource(pub ServerState);

pub struct MultiplayerSimpleNetServerPlugin {
    pub address: SocketAddr,
}

impl Plugin for MultiplayerSimpleNetServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ServerStateResource(ServerState::new()));
        app.add_systems(Update, read_on_server);

        let server = server_factory().new_server(
            enfync::builtin::native::TokioHandle::default(),
            self.address,
            AcceptorConfig::Default,
            Authenticator::None,
            ServerConfig {
                rate_limit_config: RateLimitConfig {
                    period:    Duration::from_millis(100),
                    max_count: 100,
                },
                ..default()
            },
        );

        info!("Server started on {:?}.", self.address);

        app.insert_resource(server);
    }
}

fn read_on_server(
    mut server: ResMut<Server<GameChannel>>,
    mut server_state_resource: ResMut<ServerStateResource>,
) {
    let ServerStateResource(ref mut server_state) = server_state_resource.as_mut();

    while let Some((session_id, server_event)) = server.next() {
        match server_event {
            GameServerEvent::Report(connection_report) => {
                match connection_report {
                    ServerReport::Connected(env, message) => {
                        info!("Connected {session_id} {env:?} {message:?}");
                    },
                    ServerReport::Disconnected => info!("Disconnected {session_id}"),
                }
            },
            GameServerEvent::Msg(EncodedClientMsg(message)) => {
                match bincode::deserialize::<ClientCommand>(&message) {
                    Ok(command) => {
                        info!("Received {command:?}");
                        // TODO: Only returning responses when processing a command will not allow timer-based updates. We may need to introduce an intermediate event queue for `ClientCommandWithClientId`-s where `ServerState` can publish updates.
                        let responses = server_state.process(ClientCommandWithClientId {
                            client_id: ClientId::from_u128(session_id),
                            command,
                        });
                        process_responses(server.as_mut(), responses);
                    },
                    Err(error) => {
                        error!("Failed to deserialize {message:?}: {error}");
                    },
                }
            },
            GameServerEvent::Request(token, request) => {
                error!("Unexpected request: {token:?} {request:?}");
            },
        }
    }
}

fn process_responses(
    server: &mut Server<GameChannel>,
    responses: Vec<ServerResponseWithClientIds>,
) {
    for response in responses {
        info!("Sending {response:?}...");
        for client_id in response.client_ids {
            match bincode::serialize(&response.response) {
                Ok(encoded) => {
                    server.send(client_id.as_u128(), EncodedServerMsg(encoded));
                },
                Err(error) => {
                    error!("Failed to deserialize {:?}: {error}", response.response);
                },
            }
        }
    }
}

fn server_factory() -> ServerFactory<GameChannel> {
    ServerFactory::<GameChannel>::new(env!("CARGO_PKG_VERSION"))
}
