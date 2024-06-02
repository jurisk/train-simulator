use bevy::app::App;
use bevy::prelude::{
    in_state, Commands, EventReader, EventWriter, IntoSystemConfigs, NextState, OnEnter, Plugin,
    ResMut, Resource, Update,
};
use shared_domain::game_state::GameState;
use shared_protocol::game_selection::{ClientMessage, ServerMessage};

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::map_level::terrain::TerrainPlugin;
use crate::states::ClientState;

pub mod terrain;

pub(crate) struct MapLevelPlugin;

impl Plugin for MapLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerrainPlugin);
        app.add_systems(OnEnter(ClientState::Joining), request_join_game);
        app.add_systems(
            Update,
            handle_game_joined.run_if(in_state(ClientState::Joining)),
        );
    }
}

// TODO: Move to `game` or something
#[allow(clippy::module_name_repetitions)]
#[derive(Resource)]
pub struct GameStateResource {
    game_state: GameState,
}

fn request_join_game(mut client_messages: EventWriter<ClientMessageEvent>) {
    client_messages.send(ClientMessageEvent::new(ClientMessage::JoinGame));
}

fn handle_game_joined(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_state: ResMut<NextState<ClientState>>,
    mut commands: Commands,
) {
    for message in server_messages.read() {
        match &message.message {
            ServerMessage::GameJoined { game_state } => {
                commands.insert_resource(GameStateResource {
                    game_state: game_state.clone(),
                });
                client_state.set(ClientState::Playing);
            },
        }
    }
}
