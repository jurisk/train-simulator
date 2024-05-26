use bevy::app::App;
use bevy::prelude::{
    in_state, Commands, EventReader, EventWriter, IntoSystemConfigs, NextState, OnEnter, Plugin,
    ResMut, Resource, Update,
};

use crate::communication::domain::{ClientMessage, ServerMessage};
use crate::level::domain::Level;
use crate::level::terrain::TerrainPlugin;
use crate::states::GameState;

pub mod domain;
pub mod terrain;

pub(crate) struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerrainPlugin);
        app.add_systems(OnEnter(GameState::Joining), request_join_game);
        app.add_systems(
            Update,
            handle_game_joined.run_if(in_state(GameState::Joining)),
        );
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Resource)]
pub struct LevelResource {
    level: Level,
}

fn request_join_game(mut client_messages: EventWriter<ClientMessage>) {
    client_messages.send(ClientMessage::JoinGame);
}

fn handle_game_joined(
    mut server_messages: EventReader<ServerMessage>,
    mut game_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    for message in server_messages.read() {
        match message {
            ServerMessage::GameJoined { level } => {
                commands.insert_resource(LevelResource {
                    level: level.clone(),
                });
                game_state.set(GameState::Playing);
            },
        }
    }
}
