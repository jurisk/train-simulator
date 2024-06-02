use bevy::app::App;
use bevy::prelude::{
    in_state, Commands, EventReader, EventWriter, IntoSystemConfigs, NextState, OnEnter, Plugin,
    ResMut, Resource, Update,
};

use crate::level::buildings::BuildingPlugin;
use crate::level::terrain::TerrainPlugin;
use crate::states::GameState;
use shared_domain::level::Level;
use shared_protocol::game_selection::{ClientMessage, ServerMessage};
use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};

mod buildings;
pub mod terrain;

pub(crate) struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerrainPlugin);
        app.add_plugins(BuildingPlugin);
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

fn request_join_game(mut client_messages: EventWriter<ClientMessageEvent>) {
    client_messages.send(ClientMessageEvent::new(ClientMessage::JoinGame));
}

fn handle_game_joined(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut game_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    for message in server_messages.read() {
        match &message.message {
            ServerMessage::GameJoined { level } => {
                commands.insert_resource(LevelResource {
                    level: level.clone(),
                });
                game_state.set(GameState::Playing);
            },
        }
    }
}
