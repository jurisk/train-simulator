use std::env::args;
use std::str::FromStr;

use bevy::prelude::{
    info, App, EventReader, EventWriter, FixedUpdate, Res, ResMut, Resource, Time, Update,
};
use client_graphics::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use client_graphics::game::GameLaunchParams;
use client_graphics::states::ClientState;
use client_graphics::ClientGraphicsPlugin;
use game_logic::server_state::ServerState;
use shared_domain::client_command::{AccessToken, ClientCommandWithClientId};
use shared_domain::game_time::GameTime;
use shared_domain::{ClientId, PlayerId};

#[allow(clippy::expect_used)]
fn main() {
    let args: Vec<_> = args().collect();
    let player_id = match args.get(1).cloned() {
        None => PlayerId::random(),
        Some(player_id) => PlayerId::from_str(player_id.as_str()).expect("Failed to parse UUID"),
    };

    let mut app = App::new();
    let client_id = ClientId::random();
    app.insert_resource(ClientIdResource(client_id));
    app.insert_resource(ServerStateResource(ServerState::new()));
    app.add_plugins(ClientGraphicsPlugin {
        game_launch_params: GameLaunchParams {
            player_id,
            access_token: AccessToken("valid-token".to_string()),
            game_id: None,
        },
    });
    app.insert_state(ClientState::LoggingIn);
    app.add_systems(FixedUpdate, process_messages_locally);
    app.add_systems(Update, advance_time_locally);
    app.run();
}

#[derive(Resource)]
struct ClientIdResource(ClientId);

#[derive(Resource)]
struct ServerStateResource(pub ServerState);

#[allow(clippy::needless_pass_by_value)]
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

        info!(
            "Simulating server: processing message: {:?}",
            client_command_with_client_id
        );
        let responses = server_state.process(client_command_with_client_id);
        for response in responses {
            info!("Simulating server: Got response: {:?}", response);
            if response.client_ids.contains(&client_id) {
                server_messages.send(ServerMessageEvent::new(response.response));
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn advance_time_locally(
    time: Res<Time>,
    client_id_resource: Res<ClientIdResource>,
    mut server_state_resource: ResMut<ServerStateResource>,
    mut server_messages: EventWriter<ServerMessageEvent>,
) {
    let ClientIdResource(client_id) = *client_id_resource;
    let ServerStateResource(ref mut server_state) = server_state_resource.as_mut();
    server_state.advance_times(GameTime::from_seconds(time.elapsed_seconds()));

    let sync_responses = server_state.sync_games();
    for response in sync_responses {
        info!("Simulating server: Got sync response: {:?}", response);
        if response.client_ids.contains(&client_id) {
            server_messages.send(ServerMessageEvent::new(response.response));
        }
    }
}
