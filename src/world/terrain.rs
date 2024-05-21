use bevy::app::App;
use bevy::asset::Assets;
use bevy::pbr::{AlphaMode, PbrBundle, StandardMaterial};
use bevy::prelude::shape::Plane;
use bevy::prelude::{default, Color, Commands, Cuboid, Mesh, Plugin, ResMut, Startup, Transform};

pub(crate) struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_terrain);
    }
}

fn create_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Water
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane::from_size(10.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::rgba_u8(173, 216, 230, 32),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    // Sea bottom
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane::from_size(10.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    });

    // Mountain
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
}
