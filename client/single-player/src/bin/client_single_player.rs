use bevy::prelude::AppExtStates;
use bevy::prelude::{
    debug, App, EventReader, EventWriter, FixedUpdate, Res, ResMut, Resource, Time,
};
use clap::Parser;
use client_graphics::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use client_graphics::game::GameLaunchParams;
use client_graphics::states::ClientState;
use client_graphics::ClientGraphicsPlugin;
use game_logic::server_state::ServerState;
use shared_domain::client_command::ClientCommandWithClientId;
use shared_domain::game_time::GameTime;
use shared_domain::ClientId;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[allow(clippy::struct_field_names)]
struct Args {
    #[clap(short, long)]
    player_id: Option<String>,
    #[clap(short, long)]
    map_id:    Option<String>,
    #[clap(short, long)]
    game_id:   Option<String>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start(player_id: &str, map_id: &str, game_id: &str) {
    run_with_string(player_id, map_id, game_id);
}

fn run_with_string(player_id: &str, map_id: &str, game_id: &str) {
    let access_token = "valid-token";
    let game_launch_params = GameLaunchParams::new(player_id, access_token, map_id, game_id);

    run(game_launch_params);
}

#[allow(clippy::expect_used)]
fn main() {
    let args = Args::parse();
    run_with_string(
        &args.player_id.unwrap_or_default(),
        &args.map_id.unwrap_or_default(),
        &args.game_id.unwrap_or_default(),
    );
}

fn run(game_launch_params: GameLaunchParams) {
    println!("Starting client: {game_launch_params:?}");
    let mut app = App::new();
    let client_id = ClientId::random();
    app.insert_resource(ClientIdResource(client_id));
    app.insert_resource(ServerStateResource(ServerState::new()));
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

        debug!(
            "Simulating server: processing message: {:?}",
            client_command_with_client_id
        );
        let responses = server_state.process(&client_command_with_client_id);
        for response in responses {
            debug!("Simulating server: Got response: {:?}", response);
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
        debug!("Simulating server: Got sync response: {:?}", response);
        if response.client_ids.contains(&client_id) {
            server_messages.send(ServerMessageEvent::new(response.response));
        }
    }
}
