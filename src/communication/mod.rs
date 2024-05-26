use bevy::app::App;
use bevy::prelude::{info, EventReader, EventWriter, Plugin, Update};

use crate::communication::domain::{ClientMessage, ServerMessage};
use crate::level::domain::Level;

pub mod domain;

#[allow(clippy::module_name_repetitions)]
pub struct CommunicationPlugin;

impl Plugin for CommunicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ClientMessage>();
        app.add_event::<ServerMessage>();
        app.add_systems(Update, server_stub);
    }
}

// TODO: Eventually, we should replace it with a real server communications component, but not sure which library is the best - possibly https://github.com/ukoehb/bevy_simplenet
fn server_stub(
    mut client_messages: EventReader<ClientMessage>,
    mut server_messages: EventWriter<ServerMessage>,
) {
    for message in client_messages.read() {
        match message {
            ClientMessage::JoinGame => {
                let level_json = include_str!("../../assets/levels/default.json");
                let level = serde_json::from_str::<Level>(level_json)
                    .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));

                info!("Simulating server responding to JoinGame with GameJoined");

                server_messages.send(ServerMessage::GameJoined { level });
            },
        }
    }
}
