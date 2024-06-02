#![allow(clippy::needless_pass_by_value)]

use bevy::core::Name;
use bevy::pbr::PbrBundle;
use bevy::prelude::{
    default, AssetServer, Assets, Color, Commands, Mesh, Meshable, OnEnter, Plugin, Res, ResMut,
    SceneBundle, Sphere, StandardMaterial, Transform, Vec3, Visibility,
};
use shared_domain::map_level::Height;
use shared_util::coords_xz::CoordsXZ;

use crate::game::map_level::terrain::land::logical_to_world;
use crate::game::GameStateResource;
use crate::states::ClientState;

pub(crate) struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        // TODO: All of these `setup`, including `water` and `land` should probably happen at another time point instead of entering `ClientState::Playing`
        app.add_systems(OnEnter(ClientState::Playing), setup);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_state: Res<GameStateResource>,
) {
    let vertex_coords_xz = CoordsXZ::new(3, 5);
    let height = Height(12);
    let translation = logical_to_world(
        vertex_coords_xz,
        height,
        &game_state.game_state.map_level.terrain,
    );

    // TODO: The location of the factory is off, we have to figure out how to center it properly in the tiles
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("models/factory.glb#Scene0"),
            transform: Transform {
                translation,
                scale: Vec3::new(0.079, 0.079, 0.079),
                ..default()
            },
            visibility: Visibility::Hidden, // TODO: Hiding for now as it was off anyway
            ..default()
        },
        Name::new("Test Factory"),
    ));

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
