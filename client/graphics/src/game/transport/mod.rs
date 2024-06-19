mod train;

use std::collections::HashMap;

use bevy::app::App;
use bevy::asset::Assets;
use bevy::log::error;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Commands, EventReader, FixedUpdate, Mesh, Plugin, Res, ResMut, Update};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, PlayerInfo, ServerResponse};
use shared_domain::{PlayerId, TransportInfo, TransportType};

use crate::communication::domain::ServerMessageEvent;
use crate::game::map_level::MapLevelResource;
use crate::game::transport::train::create_train;
use crate::game::PlayersInfoResource;

pub struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_transport_created);
        app.add_systems(Update, move_transports);
    }
}

fn move_transports() {
    // TODO: Implement - should we spawn each Transport as a parent entity?
}

#[allow(clippy::collapsible_match, clippy::needless_pass_by_value)]
fn handle_transport_created(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_level: Option<Res<MapLevelResource>>,
    players_info_resource: Res<PlayersInfoResource>,
) {
    let PlayersInfoResource(players_info) = players_info_resource.as_ref();

    if let Some(map_level) = map_level {
        for message in server_messages.read() {
            if let ServerResponse::Game(_game_id, game_response) = &message.response {
                if let GameResponse::TransportCreated(transport_info) = game_response {
                    create_transport(
                        transport_info,
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &map_level.map_level,
                        players_info,
                    );
                }
            }
        }
    }
}

#[allow(clippy::similar_names)]
fn create_transport(
    transport_info: &TransportInfo,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
    players_info: &HashMap<PlayerId, PlayerInfo>,
) {
    match players_info.get(&transport_info.owner_id) {
        None => {
            error!("Player with ID {:?} not found", transport_info.owner_id);
        },
        Some(player_info) => {
            match &transport_info.transport_type {
                TransportType::Train(train_components) => {
                    create_train(
                        player_info,
                        &transport_info.location,
                        train_components,
                        commands,
                        meshes,
                        materials,
                        map_level,
                    );
                },
                TransportType::RoadVehicle => {
                    todo!() // TODO: Implement
                },
                TransportType::Ship => {
                    todo!() // TODO: Implement
                },
            }
        },
    }
}
