use bevy::prelude::{App, EventReader, EventWriter, Update};
use client_graphics::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use client_graphics::ClientGraphicsPlugin;
use game_logic::logic::server_logic;
use shared_domain::PlayerId;

fn main() {
    let mut app = App::new();
    app.add_plugins(ClientGraphicsPlugin);
    app.add_systems(Update, process_messages_locally);
    app.run();
}

// TODO:    Eventually, we should also introduce a multi-player client with a real server communications component.
//          Not sure which library is the best - possibly https://github.com/ukoehb/bevy_simplenet
// TODO:    The server should likely be stateful, separate state for lobby and for each game
fn process_messages_locally(
    mut client_messages: EventReader<ClientMessageEvent>,
    mut server_messages: EventWriter<ServerMessageEvent>,
) {
    for message in client_messages.read() {
        let responses = server_logic(&message.command, PlayerId::random());
        for response in responses {
            // TODO: We are ignoring the response address
            server_messages.send(ServerMessageEvent::new(response.response));
        }
    }
}
