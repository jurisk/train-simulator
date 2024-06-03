#![allow(clippy::needless_pass_by_value)]

use bevy::core::Name;
use bevy::pbr::PbrBundle;
use bevy::prelude::{
    default, in_state, Assets, Color, Commands, EventReader, IntoSystemConfigs, Mesh, Meshable,
    Plugin, Res, ResMut, Sphere, StandardMaterial, Transform, Update, Vec3,
};
use shared_domain::game_state::GameState;
use shared_domain::server_response::{GameResponse, ServerResponse};
use shared_domain::{BuildingInfo, BuildingType, TrackType};
use shared_util::coords_xz::CoordsXZ;

use crate::communication::domain::ServerMessageEvent;
use crate::game::map_level::terrain::land::logical_to_world;
use crate::game::GameStateResource;
use crate::states::ClientState;

pub(crate) struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            Update,
            handle_game_state_responses.run_if(in_state(ClientState::Playing)), // Not sure about race conditions
        );
    }
}

#[allow(clippy::collapsible_match)]
fn handle_game_state_responses(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_state: Res<GameStateResource>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(game_response) = &message.response {
            match game_response {
                GameResponse::BuildingBuilt(building_info) => {
                    create_building(
                        building_info,
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &game_state.game_state,
                    );
                },

                GameResponse::State(game_state) => {
                    for building_info in &game_state.buildings {
                        create_building(
                            building_info,
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            game_state,
                        );
                    }
                },
            }
        }
    }
}

fn create_building(
    building_info: &BuildingInfo,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_state: &GameState,
) {
    match &building_info.building_type {
        BuildingType::Track(track_type) => {
            create_track(
                commands,
                meshes,
                materials,
                game_state,
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
    game_state: &GameState,
    vertex_coords_xz: CoordsXZ,
    track_type: TrackType,
) {
    let terrain = &game_state.map_level.terrain;
    let height = terrain.vertex_heights[&vertex_coords_xz];
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
