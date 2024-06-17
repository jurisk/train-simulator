#![allow(clippy::expect_used)]

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::Body;
use axum::Router;
use bevy::prelude::{
    debug, default, error, info, App, Event, EventReader, EventWriter, FixedUpdate,
    IntoSystemConfigs, Plugin, ResMut, Resource,
};
use bevy_simplenet::{
    AcceptorConfig, Authenticator, RateLimitConfig, Server, ServerConfig, ServerEventFrom,
    ServerFactory, ServerReport,
};
use game_logic::server_state::ServerState;
use networking_shared::{EncodedClientMsg, EncodedServerMsg, GameChannel};
use shared_domain::client_command::{ClientCommand, ClientCommandWithClientId};
use shared_domain::ClientId;

pub type GameServerEvent = ServerEventFrom<GameChannel>;

#[derive(Resource)]
struct ServerStateResource(pub ServerState);

pub struct MultiplayerSimpleNetServerPlugin {
    // TODO: Can we do this simpler?
    pub router:  Arc<Mutex<Router<(), Body>>>,
    pub address: SocketAddr,
}

impl Plugin for MultiplayerSimpleNetServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ServerStateResource(ServerState::new()));
        app.add_systems(FixedUpdate, read_on_server);
        // Fair warning - there are interesting race conditions that can happen in `bevy_simplenet`:
        // - Messages will be silently dropped if there are unconsumed connection reports for that client.
        app.add_systems(
            FixedUpdate,
            process_client_command_with_client_id_events.after(read_on_server),
        );
        app.add_event::<ClientCommandWithClientIdEvent>();

        let router = self.router.clone();

        let server = {
            let mut router = router.lock().expect("Locking the router failed");
            let router = &mut *router;
            server_factory().new_server_with_router(
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
                router.clone(),
            )
        };

        info!("Server started on {:?}.", self.address);

        app.insert_resource(server);
    }
}

#[derive(Event)]
struct ClientCommandWithClientIdEvent(ClientCommandWithClientId);

fn read_on_server(
    mut server: ResMut<Server<GameChannel>>,
    mut client_command_with_client_id_events: EventWriter<ClientCommandWithClientIdEvent>,
) {
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
                        let client_command_with_client_id_event =
                            ClientCommandWithClientIdEvent(ClientCommandWithClientId {
                                client_id: ClientId::from_u128(session_id),
                                command,
                            });
                        client_command_with_client_id_events
                            .send(client_command_with_client_id_event);
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

// TODO: Only returning responses when processing a command will not allow timer-based updates. We may need to introduce an intermediate event queue for `ClientCommandWithClientId`-s where `ServerState` can publish updates.
#[allow(clippy::needless_pass_by_value)]
fn process_client_command_with_client_id_events(
    mut server_state_resource: ResMut<ServerStateResource>,
    server: ResMut<Server<GameChannel>>,
    mut client_command_with_client_id_events: EventReader<ClientCommandWithClientIdEvent>,
) {
    let ServerStateResource(ref mut server_state) = server_state_resource.as_mut();

    for ClientCommandWithClientIdEvent(client_command_with_client_id) in
        client_command_with_client_id_events.read()
    {
        debug!("Picked up {client_command_with_client_id:?}...");
        let responses = server_state.process(client_command_with_client_id.clone());

        for response in responses {
            for client_id in &*response.client_ids {
                match bincode::serialize(&response.response) {
                    Ok(encoded) => {
                        info!("Sending {response:?} to {client_id:?}...");
                        server.send(client_id.as_u128(), EncodedServerMsg(encoded));
                    },
                    Err(error) => {
                        error!("Failed to deserialize {:?}: {error}", response.response);
                    },
                }
            }
        }
    }
}

fn server_factory() -> ServerFactory<GameChannel> {
    ServerFactory::<GameChannel>::new(env!("CARGO_PKG_VERSION"))
}
