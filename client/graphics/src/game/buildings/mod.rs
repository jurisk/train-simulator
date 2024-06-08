#![allow(clippy::needless_pass_by_value)]

use bevy::core::Name;
use bevy::pbr::PbrBundle;
use bevy::prelude::{
    default, Assets, Color, Commands, EventReader, Mesh, Meshable, Plugin, Res, ResMut, Sphere,
    StandardMaterial, Transform, Update, Vec3,
};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, ServerResponse};
use shared_domain::{BuildingInfo, BuildingType, TrackType, VertexCoordsXZ};

use crate::communication::domain::ServerMessageEvent;
use crate::game::map_level::terrain::land::logical_to_world;
use crate::game::map_level::MapLevelResource;

pub(crate) struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, handle_building_built);
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
    vertex_coords_xz: VertexCoordsXZ,
    track_type: TrackType,
) {
    let terrain = &map_level.terrain;
    let height = terrain.vertex_heights[&vertex_coords_xz.into()];
    let translation = logical_to_world(vertex_coords_xz, height, terrain);

    let color = match track_type {
        TrackType::NorthSouth => Color::RED,
        TrackType::EastWest => Color::BLUE,
    };

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
        Name::new(format!(
            "Track {track_type:?} at {vertex_coords_xz:?} {height:?}"
        )),
    ));
}
