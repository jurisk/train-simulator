#![allow(clippy::needless_pass_by_value)]

use bevy::core::Name;
use bevy::pbr::PbrBundle;
use bevy::prelude::{
    default, in_state, Assets, Color, Commands, EventReader, IntoSystemConfigs, Mesh, Meshable,
    Plugin, Res, ResMut, Sphere, StandardMaterial, Transform, Update, Vec3,
};
use shared_domain::map_level::Height;
use shared_domain::BuildingType;
use shared_protocol::server_response::{GameResponse, ServerResponse};
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

fn handle_game_state_responses(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_state: Res<GameStateResource>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(game_response) = &message.response {
            if let GameResponse::BuildingBuilt(building_info) = game_response {
                if let BuildingType::Track(_track_type) = &building_info.building_type {
                    crate_track(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &game_state,
                        building_info.vertex_coords_xz,
                    );
                }
            }
        }
    }
}

fn crate_track(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_state: &Res<GameStateResource>,
    vertex_coords_xz: CoordsXZ,
) {
    let height = Height(12);
    let translation = logical_to_world(
        vertex_coords_xz,
        height,
        &game_state.game_state.map_level.terrain,
    );

    commands.spawn((
        PbrBundle {
            transform: Transform {
                translation,
                scale: Vec3::new(0.1, 0.1, 0.1),
                ..default()
            },
            material: materials.add(Color::RED),
            mesh: meshes.add(Sphere::default().mesh().uv(32, 18)),
            ..default()
        },
        Name::new(format!(
            "Test Sphere at vertex {vertex_coords_xz:?} {height:?}"
        )),
    ));
}
