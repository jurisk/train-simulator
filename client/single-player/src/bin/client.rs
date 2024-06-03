use bevy::prelude::{App, EventReader, EventWriter, Res, Resource, Update};
use client_graphics::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use client_graphics::ClientGraphicsPlugin;
use game_logic::logic::server_logic;
use shared_domain::ClientId;
use shared_protocol::client_command::ClientCommandWithClientId;

fn main() {
    let mut app = App::new();
    let client_id = ClientId::random();
    app.insert_resource(ClientIdResource(client_id));
    app.add_plugins(ClientGraphicsPlugin);
    app.add_systems(Update, process_messages_locally);
    app.run();
}

#[derive(Resource)]
struct ClientIdResource(ClientId);

// TODO:    Eventually, we should also introduce a multi-player client with a real server communications component.
//          Not sure which library is the best - possibly https://github.com/ukoehb/bevy_simplenet ?
//          Or maybe start with Renet that you already know, but avoid having a dependency to it anywhere but in `client/multi-player` and `server`?
// TODO:    The server should likely be stateful, separate state for lobby and for each game
#[allow(clippy::needless_pass_by_value)]
fn process_messages_locally(
    client_id_resource: Res<ClientIdResource>,
    mut client_messages: EventReader<ClientMessageEvent>,
    mut server_messages: EventWriter<ServerMessageEvent>,
) {
    for message in client_messages.read() {
        let client_command_with_client_id =
            ClientCommandWithClientId::new(client_id_resource.0, message.command.clone());
        let responses = server_logic(client_command_with_client_id);
        for response in responses {
            // TODO: We are ignoring the response address
            server_messages.send(ServerMessageEvent::new(response.response));
        }
    }
}
