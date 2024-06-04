use bevy::app::{App, Update};
use bevy::prelude::{Commands, EventReader, NextState, Plugin, ResMut, Resource};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, ServerResponse};

use crate::communication::domain::ServerMessageEvent;
use crate::game::map_level::terrain::TerrainPlugin;
use crate::states::ClientState;

pub mod terrain;

#[allow(clippy::module_name_repetitions)]
#[derive(Resource)]
pub struct MapLevelResource {
    pub map_level: MapLevel,
}

pub(crate) struct MapLevelPlugin;

impl Plugin for MapLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerrainPlugin);
        app.add_systems(Update, handle_map_level_updated);
    }
}

// TODO: How does `terrain` differ from `map_level`? What about trees? Is it `MapLevel`? Is it `Buildings`?

#[allow(clippy::collapsible_match)]
fn handle_map_level_updated(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_state: ResMut<NextState<ClientState>>,
    mut commands: Commands,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(game_response) = &message.response {
            if let GameResponse::MapLevelUpdated(map_level) = game_response {
                commands.insert_resource(MapLevelResource {
                    map_level: map_level.clone(),
                });
                client_state.set(ClientState::Playing);
            }
        }
    }
}
