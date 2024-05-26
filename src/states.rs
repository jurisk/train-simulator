use bevy::prelude::States;

#[derive(States, Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    #[default]
    Joining,
    Playing,
}
