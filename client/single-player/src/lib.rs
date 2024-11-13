use bevy::app::{App, FixedUpdate};
use bevy::log::debug;
use bevy::prelude::{AppExtStates, EventReader, EventWriter, Res, ResMut, Resource, Time};
use client_graphics::ClientGraphicsPlugin;
use client_graphics::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use client_graphics::game::GameLaunchParams;
use client_graphics::states::ClientState;
use game_logic::server_state::ServerState;
use shared_domain::ClientId;
use shared_domain::client_command::ClientCommandWithClientId;
use shared_domain::game_time::GameTime;
use shared_domain::metrics::NoopMetrics;

pub fn run(game_launch_params: GameLaunchParams) {
    println!("Starting client: {game_launch_params:?}");
    let mut app = App::new();
    let client_id = ClientId::random();
    app.insert_resource(ClientIdResource(client_id));
    app.insert_resource(ServerStateResource(ServerState::new(true)));
    app.add_plugins(ClientGraphicsPlugin { game_launch_params });
    app.insert_state(ClientState::LoggingIn);
    app.add_systems(FixedUpdate, process_messages_locally);
    app.add_systems(FixedUpdate, advance_time_locally);
    app.run();
}

#[derive(Resource)]
struct ClientIdResource(ClientId);

#[derive(Resource)]
struct ServerStateResource(pub ServerState);

#[expect(clippy::needless_pass_by_value)]
fn process_messages_locally(
    client_id_resource: Res<ClientIdResource>,
    mut server_state_resource: ResMut<ServerStateResource>,
    mut client_messages: EventReader<ClientMessageEvent>,
    mut server_messages: EventWriter<ServerMessageEvent>,
) {
    let ClientIdResource(client_id) = *client_id_resource;
    let ServerStateResource(ref mut server_state) = server_state_resource.as_mut();

    for message in client_messages.read() {
        let client_command_with_client_id =
            ClientCommandWithClientId::new(client_id, message.command.clone());

        debug!(
            "Simulating server: processing message: {:?}",
            client_command_with_client_id
        );
        let responses = server_state.process(&client_command_with_client_id);
        for response in responses {
            debug!("Simulating server: Got response: {:?}", response);
            // Later: This is somewhat of a hack, because if AIs are sending messages on behalf of various players, then we should ideally distinguish which player each server response is targeted at. In some sense, each client can receive messages for multiple players, and those get routed to the right AI. But this is a lot of complication, so we should start fixing it only if the current approach doesn't work.
            if response.client_ids.contains(&client_id) {
                server_messages.send(ServerMessageEvent::new(response.response));
            }
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
fn advance_time_locally(
    time: Res<Time>,
    client_id_resource: Res<ClientIdResource>,
    mut server_state_resource: ResMut<ServerStateResource>,
    mut server_messages: EventWriter<ServerMessageEvent>,
) {
    let ClientIdResource(client_id) = *client_id_resource;
    let ServerStateResource(ref mut server_state) = server_state_resource.as_mut();
    server_state.advance_times(
        GameTime::from_seconds(time.elapsed_seconds()),
        &NoopMetrics::default(),
    );

    let sync_responses = server_state.sync_games();
    for response in sync_responses {
        debug!("Simulating server: Got sync response: {:?}", response);
        if response.client_ids.contains(&client_id) {
            server_messages.send(ServerMessageEvent::new(response.response));
        }
    }
}
