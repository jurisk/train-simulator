use bevy::prelude::States;

// Later: Consider having an "assets loaded" state as you have some likely race conditions in FPS counter.
#[derive(States, Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    #[default]
    Joining,
    Playing,
}
