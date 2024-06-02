use bevy::app::App;
use bevy::prelude::{info, EventReader, EventWriter, Plugin, Update};
use shared_protocol::game_selection::{ClientMessage, ServerMessage};

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use shared_domain::level::Level;
pub mod domain;

#[allow(clippy::module_name_repetitions)]
pub struct CommunicationPlugin;

impl Plugin for CommunicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ClientMessageEvent>();
        app.add_event::<ServerMessageEvent>();
        app.add_systems(Update, process_messages);
    }
}

fn process_messages(
    mut client_messages: EventReader<ClientMessageEvent>,
    mut server_messages: EventWriter<ServerMessageEvent>,
) {
    for message in client_messages.read() {
        let responses = server_stub(&message.message);
        for response in responses {
            server_messages.send(ServerMessageEvent::new(response));
        }
    }
}

// TODO: Move this to the `single-player` module
// TODO: Eventually, we should replace it with a real server communications component, but not sure which library is the best - possibly https://github.com/ukoehb/bevy_simplenet
fn server_stub(client_message: &ClientMessage) -> Vec<ServerMessage> {
    match client_message {
        ClientMessage::JoinGame => {
            let level_json = include_str!("../../assets/levels/default.json");
            let level = serde_json::from_str::<Level>(level_json)
                .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));

            assert!(level.is_valid());

            info!("Simulating server responding to JoinGame with GameJoined");

            vec![ServerMessage::GameJoined { level }]
        },
    }
}