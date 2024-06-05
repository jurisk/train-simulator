use bevy::prelude::Event;
use shared_domain::client_command::ClientCommand;
use shared_domain::server_response::ServerResponse;

#[derive(Event, Debug)]
pub struct ClientMessageEvent {
    pub command: ClientCommand,
}

impl ClientMessageEvent {
    #[must_use]
    pub fn new(command: ClientCommand) -> Self {
        Self { command }
    }
}

#[derive(Event)]
pub struct ServerMessageEvent {
    pub(crate) response: ServerResponse,
}

impl ServerMessageEvent {
    #[must_use]
    pub fn new(response: ServerResponse) -> Self {
        Self { response }
    }
}
