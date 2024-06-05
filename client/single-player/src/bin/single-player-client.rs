use bevy::log::info;
use bevy::prelude::{App, EventReader, EventWriter, Res, ResMut, Resource, Update};
use client_graphics::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use client_graphics::states::ClientState;
use client_graphics::ClientGraphicsPlugin;
use game_logic::server_state::ServerState;
use shared_domain::client_command::ClientCommandWithClientId;
use shared_domain::ClientId;

fn main() {
    let mut app = App::new();
    let client_id = ClientId::random();
    app.insert_resource(ClientIdResource(client_id));
    app.insert_resource(ServerStateResource(ServerState::new()));
    app.insert_state(ClientState::JoiningGame);
    app.add_plugins(ClientGraphicsPlugin);
    app.add_systems(Update, process_messages_locally);
    app.run();
}

#[derive(Resource)]
struct ClientIdResource(ClientId);

#[derive(Resource)]
struct ServerStateResource(pub ServerState);

// TODO:    Eventually, we should also introduce a multi-player client with a real server communications component.
//          Not sure which library is the best - possibly https://github.com/ukoehb/bevy_simplenet ?
//          Or maybe start with Renet that you already know, but avoid having a dependency to it anywhere but in `client/multi-player` and `server`?
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

        info!("Processing message: {:?}", client_command_with_client_id);
        let responses = server_state.process(client_command_with_client_id);
        for response in responses {
            info!("Got response: {:?}", response);
            if response.client_ids.contains(&client_id) {
                server_messages.send(ServerMessageEvent::new(response.response));
            }
        }
    }
}
