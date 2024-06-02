use bevy::prelude::Event;
use shared_protocol::game_selection::{ClientMessage, ServerMessage};

#[derive(Event)]
pub struct ClientMessageEvent {
    pub(crate) message: ClientMessage,
}

impl ClientMessageEvent {
    pub fn new(message: ClientMessage) -> Self {
        Self { message }
    }
}

#[derive(Event)]
pub struct ServerMessageEvent {
    pub(crate) message: ServerMessage,
}

impl ServerMessageEvent {
    pub fn new(message: ServerMessage) -> Self {
        Self { message }
    }
}
