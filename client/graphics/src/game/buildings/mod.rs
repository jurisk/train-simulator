#![allow(clippy::needless_pass_by_value, clippy::collapsible_match)]

use bevy::core::Name;
use bevy::pbr::PbrBundle;
use bevy::prelude::{
    default, Assets, Color, Commands, EventReader, EventWriter, Mesh, Meshable, Plugin, Res,
    ResMut, Sphere, StandardMaterial, Transform, Update, Vec3,
};
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, ServerResponse};
use shared_domain::{BuildingId, BuildingInfo, BuildingType, TrackType, VertexCoordsXZ};

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::map_level::terrain::land::logical_to_world;
use crate::game::map_level::MapLevelResource;
use crate::game::PlayerIdResource;

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
) {
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
                    );
                }
            }
        }
    }
}

fn create_building(
    building_info: &BuildingInfo,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
) {
    match &building_info.building_type {
        BuildingType::Track(track_type) => {
            create_track(
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
}

fn create_track(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
    north_west_vertex_xz: VertexCoordsXZ,
    track_type: TrackType,
) {
    let terrain = &map_level.terrain;
    let translation = logical_to_world(north_west_vertex_xz, terrain);

    // TODO: Take color from the player who owns the track!
    let color = match track_type {
        TrackType::NorthSouth => Color::RED,
        TrackType::EastWest => Color::BLUE,
    };

    // TODO: Track shape instead of a Sphere!
    commands.spawn((
        PbrBundle {
            transform: Transform {
                translation,
                scale: Vec3::new(0.2, 0.2, 0.2),
                ..default()
            },
            material: materials.add(color),
            mesh: meshes.add(Sphere::default().mesh().uv(32, 18)),
            ..default()
        },
        Name::new(format!("Track {track_type:?} at {north_west_vertex_xz:?}")),
    ));
}
