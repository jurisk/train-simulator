use bevy::app::App;
use bevy::asset::Assets;
use bevy::pbr::{AlphaMode, PbrBundle, StandardMaterial};
use bevy::prelude::shape::Plane;
use bevy::prelude::{default, Color, Commands, Cuboid, Mesh, Plugin, ResMut, Startup, Transform};
use rand::Rng;

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
    let size = 10.0;

    // Water
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane::from_size(size)),
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
        mesh: meshes.add(Plane::from_size(size)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(0.0, -1.0, 0.0),
        ..default()
    });

    // Mountains
    let mut rng = rand::thread_rng();
    let n = 6;
    for x in 0..n {
        for y in 0..n {
            let height = rng.gen_range(0..4) as u8 as f32;
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(size / n as f32, height, size / n as f32)),
                material: materials.add(Color::rgb(rng.gen(), rng.gen(), rng.gen())),
                transform: Transform::from_xyz(
                    size * (((x as f32 + 0.5) / n as f32) - 0.5),
                    height / 2.0,
                    size * (((y as f32 + 0.5) / n as f32) - 0.5),
                ),
                ..default()
            });
        }
    }
}
