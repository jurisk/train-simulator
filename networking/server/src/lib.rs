#![allow(clippy::expect_used)]

pub mod metrics;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::Router;
use bevy::prelude::{
    App, Event, EventReader, EventWriter, FixedUpdate, IntoSystemConfigs, Plugin, Res, ResMut,
    Resource, Time, default,
};
use bevy_simplenet::{
    AcceptorConfig, Authenticator, RateLimitConfig, Server, ServerConfig, ServerEventFrom,
    ServerFactory, ServerReport,
};
use game_logic::server_state::ServerState;
use log::{Level, debug, error, info, log};
use networking_shared::{EncodedClientMsg, EncodedServerMsg, GameChannel};
use shared_domain::ClientId;
use shared_domain::client_command::{ClientCommand, ClientCommandWithClientId};
use shared_domain::game_time::GameTimeDiff;
use shared_domain::server_response::{GameResponse, ServerResponse, ServerResponseWithClientIds};
use web_time::Duration;

use crate::metrics::PrometheusMetrics;

pub type GameServerEvent = ServerEventFrom<GameChannel>;

#[derive(Resource)]
struct ServerStateResource(pub ServerState);

pub struct MultiplayerSimpleNetServerPlugin {
    pub router:  Arc<Mutex<Router>>,
    pub address: SocketAddr,
}

impl Plugin for MultiplayerSimpleNetServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ServerStateResource(ServerState::new(false)));
        app.add_systems(FixedUpdate, read_on_server);
        // Fair warning - there are interesting race conditions that can happen in `bevy_simplenet`:
        // - Messages will be silently dropped if there are unconsumed connection reports for that client.
        app.add_systems(
            FixedUpdate,
            process_client_command_with_client_id_events.after(read_on_server),
        );
        app.add_event::<ClientCommandWithClientIdEvent>();

        let router = self.router.lock().expect("Locking the router failed");

        let server = {
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
                            ClientCommandWithClientIdEvent(ClientCommandWithClientId::new(
                                ClientId::from_u128(session_id),
                                command,
                            ));
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

// TODO: Server should publish updates so that discrepancies in movement between server and client can be reconciled by the clients
#[expect(clippy::needless_pass_by_value)]
fn process_client_command_with_client_id_events(
    mut server_state_resource: ResMut<ServerStateResource>,
    server: ResMut<Server<GameChannel>>,
    mut client_command_with_client_id_events: EventReader<ClientCommandWithClientIdEvent>,
    time: Res<Time>,
    metrics: Res<PrometheusMetrics>,
) {
    let ServerStateResource(ref mut server_state) = server_state_resource.as_mut();

    for ClientCommandWithClientIdEvent(client_command_with_client_id) in
        client_command_with_client_id_events.read()
    {
        debug!("Picked up {client_command_with_client_id:?}...");
        let responses = server_state.process(client_command_with_client_id);

        for response in responses {
            send_responses_to_clients(server.as_ref(), &response);
        }
    }

    server_state.advance_time_diffs(
        GameTimeDiff::from_seconds(time.delta_seconds()),
        metrics.as_ref(),
    );
    for response in server_state.sync_games() {
        send_responses_to_clients(server.as_ref(), &response);
    }
}

fn send_responses_to_clients(server: &Server<GameChannel>, response: &ServerResponseWithClientIds) {
    let log_level = if matches!(
        response.response,
        ServerResponse::Game(_, GameResponse::DynamicInfosSync(_, _, _, _, _, _, _))
    ) {
        Level::Trace
    } else {
        Level::Info
    };
    log!(log_level, "Sending {response:?}...");

    for client_id in &*response.client_ids {
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

fn server_factory() -> ServerFactory<GameChannel> {
    ServerFactory::<GameChannel>::new(env!("CARGO_PKG_VERSION"))
}
