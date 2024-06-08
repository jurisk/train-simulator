#![allow(clippy::needless_pass_by_value, clippy::collapsible_match)]

use std::collections::HashMap;

use bevy::core::Name;
use bevy::pbr::PbrBundle;
use bevy::prelude::{
    default, error, Assets, Color, Commands, Cuboid, EventReader, EventWriter, Mesh, Plugin, Quat,
    Res, ResMut, StandardMaterial, Transform, Update, Vec3,
};
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, PlayerInfo, ServerResponse};
use shared_domain::{BuildingId, BuildingInfo, BuildingType, PlayerId, TrackType, VertexCoordsXZ};
use shared_util::direction_xz::DirectionXZ;

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::map_level::terrain::land::logical_to_world;
use crate::game::map_level::MapLevelResource;
use crate::game::{PlayerIdResource, PlayersInfoResource};

pub(crate) struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, handle_building_built);
        app.add_systems(Update, handle_game_map_level_provided_for_testing);
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
                // TODO: Everyone will now build the test track on the same spot... That's weird. Make it random or have each find a free spot?
                let initial_buildings = vec![
                    BuildingInfo {
                        owner_id:             player_id,
                        building_id:          BuildingId::random(),
                        north_west_vertex_xz: VertexCoordsXZ::from_usizes(10, 10),
                        building_type:        BuildingType::Track(TrackType::EastWest),
                    },
                    BuildingInfo {
                        owner_id:             player_id,
                        building_id:          BuildingId::random(),
                        north_west_vertex_xz: VertexCoordsXZ::from_usizes(3, 5),
                        building_type:        BuildingType::Track(TrackType::NorthSouth),
                    },
                ];

                for building in initial_buildings {
                    client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                        *game_id,
                        GameCommand::BuildBuilding(building),
                    )));
                }
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
                if let GameResponse::BuildingBuilt(building_info) = game_response {
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

#[allow(clippy::similar_names)]
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
                    create_track(
                        player_info,
                        commands,
                        meshes,
                        materials,
                        map_level,
                        building_info.north_west_vertex_xz,
                        *track_type,
                    );
                },
                BuildingType::Production(_) => {}, // TODO: Implement
            }
        },
    }
}

const RAIL_DIAMETER: f32 = 0.1;

#[allow(clippy::similar_names)]
fn create_track(
    player_info: &PlayerInfo,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
    north_west_vertex_xz: VertexCoordsXZ,
    track_type: TrackType,
) {
    let terrain = &map_level.terrain;

    let north_east_vertex_xz = north_west_vertex_xz + DirectionXZ::East;
    let south_east_vertex_xz = north_east_vertex_xz + DirectionXZ::South;
    let south_west_vertex_xz = north_west_vertex_xz + DirectionXZ::South;

    let nw = logical_to_world(north_west_vertex_xz, terrain);
    let ne = logical_to_world(north_east_vertex_xz, terrain);
    let se = logical_to_world(south_east_vertex_xz, terrain);
    let sw = logical_to_world(south_west_vertex_xz, terrain);

    let colour = player_info.colour;
    let color = Color::rgb_u8(colour.r, colour.g, colour.b);

    // TODO: Two tracks, and going in the right direction!
    spawn_rail(
        nw,
        se,
        color,
        commands,
        meshes,
        materials,
        format!("Track #1 {track_type:?} at {north_west_vertex_xz:?}"),
    );
    spawn_rail(
        ne,
        sw,
        color,
        commands,
        meshes,
        materials,
        format!("Track #2 {track_type:?} at {north_west_vertex_xz:?}"),
    );
}

fn spawn_rail(
    a: Vec3,
    b: Vec3,
    color: Color,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    name: String,
) {
    let direction = b - a;
    let length = direction.length();
    let direction = direction.normalize();

    commands.spawn((
        PbrBundle {
            transform: Transform {
                translation: a + direction * length / 2.0,
                rotation:    Quat::from_rotation_arc(Vec3::Z, direction),
                scale:       Vec3::new(RAIL_DIAMETER, RAIL_DIAMETER, length),
            },
            material: materials.add(color),
            mesh: meshes.add(Mesh::from(Cuboid::default())),
            ..default()
        },
        Name::new(name),
    ));
}
