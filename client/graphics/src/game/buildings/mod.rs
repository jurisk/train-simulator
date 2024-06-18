#![allow(clippy::needless_pass_by_value, clippy::collapsible_match)]

pub mod tracks;

use std::collections::HashMap;

use bevy::prelude::{
    error, Assets, Commands, EventReader, EventWriter, FixedUpdate, Mesh, Plugin, Res, ResMut,
    StandardMaterial, Update,
};
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, PlayerInfo, ServerResponse};
use shared_domain::{
    BuildingId, BuildingInfo, BuildingType, PlayerId, TileCoordsXZ, TileCoverage, TrackType,
    VehicleId, VehicleInfo, VehicleType,
};
use shared_util::direction_xz::DirectionXZ;

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::buildings::tracks::{build_track_when_mouse_released, create_track};
use crate::game::map_level::MapLevelResource;
use crate::game::{PlayerIdResource, PlayersInfoResource};

pub(crate) struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(FixedUpdate, handle_building_built);
        app.add_systems(FixedUpdate, handle_game_map_level_provided_for_testing);
        app.add_systems(Update, build_track_when_mouse_released);
    }
}

// Later: Remove this, this is only for testing
fn handle_game_map_level_provided_for_testing(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    player_id_resource: Res<PlayerIdResource>,
) {
    let PlayerIdResource(player_id) = *player_id_resource;
    for message in server_messages.read() {
        if let ServerResponse::Game(game_id, game_response) = &message.response {
            if let GameResponse::MapLevelProvided(_map_level) = game_response {
                // Later: Could add nicer junctions. See https://wiki.openttd.org/en/Community/Junctionary/Basic%204-Way%20Junction for example.
                // TODO: This is debug-only and to be removed - move this to actually use the "save game" concept instead
                let test_track = vec![
                    ((49, 43), TrackType::SouthWest),
                    ((48, 43), TrackType::EastWest),
                    ((48, 43), TrackType::NorthEast),
                    ((48, 43), TrackType::NorthWest),
                    ((48, 42), TrackType::NorthSouth),
                    ((48, 41), TrackType::NorthSouth),
                    ((48, 41), TrackType::EastWest),
                    ((48, 41), TrackType::NorthEast),
                    ((48, 41), TrackType::NorthWest),
                    ((48, 41), TrackType::SouthEast),
                    ((48, 41), TrackType::SouthWest),
                    ((47, 41), TrackType::EastWest),
                    ((48, 40), TrackType::NorthSouth),
                    ((49, 41), TrackType::EastWest),
                    ((47, 43), TrackType::EastWest),
                    ((46, 43), TrackType::SouthEast),
                    ((46, 44), TrackType::NorthSouth),
                    ((46, 45), TrackType::NorthSouth),
                    ((46, 46), TrackType::NorthSouth),
                    ((46, 47), TrackType::NorthSouth),
                    ((46, 48), TrackType::NorthSouth),
                    ((46, 49), TrackType::NorthSouth),
                    ((46, 50), TrackType::NorthSouth),
                    ((46, 51), TrackType::NorthSouth),
                    ((46, 52), TrackType::NorthEast),
                    ((47, 52), TrackType::EastWest),
                    ((48, 52), TrackType::EastWest),
                    ((49, 52), TrackType::NorthWest),
                    ((49, 44), TrackType::NorthSouth),
                    ((49, 45), TrackType::NorthSouth),
                    ((49, 46), TrackType::NorthSouth),
                    ((49, 47), TrackType::NorthSouth),
                    ((49, 48), TrackType::NorthSouth),
                    ((49, 49), TrackType::NorthSouth),
                    ((49, 50), TrackType::NorthSouth),
                    ((49, 51), TrackType::NorthSouth),
                ];

                let mut initial_buildings = vec![];
                for ((x, z), track_type) in test_track {
                    let building_info = BuildingInfo {
                        owner_id:      player_id,
                        building_id:   BuildingId::random(),
                        covers_tiles:  TileCoverage::Single(TileCoordsXZ::from_usizes(x, z)),
                        building_type: BuildingType::Track(track_type),
                    };
                    initial_buildings.push(building_info);
                }

                // TODO: This will be overlapping with other players' purchased vehicles, but this may be good for testing anyway. Improve the server so that it rejects such overlaps.
                client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                    *game_id,
                    GameCommand::PurchaseVehicle(VehicleInfo {
                        vehicle_id:   VehicleId::random(),
                        owner_id:     player_id,
                        direction:    DirectionXZ::North,
                        location:     TileCoordsXZ::from_usizes(46, 48),
                        vehicle_type: VehicleType::TrainEngine,
                    }),
                )));

                client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                    *game_id,
                    GameCommand::BuildBuildings(initial_buildings),
                )));
            }
        }
    }
}

#[allow(clippy::collapsible_match)]
fn handle_building_built(
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
                if let GameResponse::BuildingsBuilt(building_infos) = game_response {
                    for building_info in building_infos {
                        create_building(
                            building_info,
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
}

#[allow(clippy::similar_names, clippy::match_same_arms)]
fn create_building(
    building_info: &BuildingInfo,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
    players_info: &HashMap<PlayerId, PlayerInfo>,
) {
    match players_info.get(&building_info.owner_id) {
        None => {
            error!("Player with ID {:?} not found", building_info.owner_id);
        },
        Some(player_info) => {
            match &building_info.building_type {
                BuildingType::Track(track_type) => {
                    if let TileCoverage::Single(tile) = &building_info.covers_tiles {
                        create_track(
                            player_info,
                            commands,
                            meshes,
                            materials,
                            map_level,
                            *tile,
                            *track_type,
                        );
                    } else {
                        error!("Multi-tile track not supported");
                    }
                },
                BuildingType::Production(_) => {}, // TODO: Implement
                BuildingType::Station(_) => {},    // TODO: Implement
            }
        },
    }
}
