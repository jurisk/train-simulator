#![allow(clippy::needless_pass_by_value)]

use bevy::core::Name;
use bevy::pbr::PbrBundle;
use bevy::prelude::{
    default, AssetServer, Assets, Color, Commands, Mesh, Meshable, Plugin, Res, ResMut,
    SceneBundle, Sphere, StandardMaterial, Startup, Transform, Vec3,
};

pub(crate) struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // TODO: The location of the factory is off, we have to figure out how to center it properly in the tiles
    let translation = Vec3::new(16.0 * (100.0 / 99.0), 9.0, 0.0);
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("models/factory.glb#Scene0"),
            transform: Transform {
                translation,
                scale: Vec3::new(0.079, 0.079, 0.079),
                ..default()
            },
            ..default()
        },
        Name::new("Test Factory"),
    ));

    commands.spawn(PbrBundle {
        transform: Transform {
            translation,
            ..default()
        },
        material: materials.add(Color::RED),
        mesh: meshes.add(Sphere::default().mesh().uv(32, 18)),
        ..default()
    });
}
