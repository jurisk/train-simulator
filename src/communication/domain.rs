use bevy::prelude::Event;

use crate::level::domain::Level;

#[derive(Event)]
pub enum ClientMessage {
    JoinGame,
}

#[derive(Event)]
pub enum ServerMessage {
    GameJoined { level: Level },
}
