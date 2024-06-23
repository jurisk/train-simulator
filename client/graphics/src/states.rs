use bevy::prelude::States;

// Later: Consider having an "assets loaded" state as you have some likely race conditions in FPS counter.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ClientState {
    ConnectingToServer,
    LoggingIn,
    JoiningGame,
    Playing,
}
