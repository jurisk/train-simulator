use bevy::app::App;
use bevy::asset::Assets;
use bevy::pbr::{AlphaMode, PbrBundle, StandardMaterial};
#[allow(deprecated)]
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
    let sea_depth = 2.5;

    // Water
    #[allow(deprecated)]
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane::from_size(size)),
        material: materials.add(StandardMaterial {
            base_color: Color::rgba_u8(173, 216, 230, 64),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    // Mountains
    let mut rng = rand::thread_rng();
    let n = 6u8;
    for x in 0 .. n {
        for y in 0 .. n {
            let x = f32::from(x);
            let y = f32::from(y);
            let n = f32::from(n);
            let height = f32::from(rng.gen_range(1u8 ..= 5u8));
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(size / n, height, size / n)),
                material: materials.add(Color::rgb(rng.gen(), rng.gen(), rng.gen())),
                transform: Transform::from_xyz(
                    size * (((x + 0.5) / n) - 0.5),
                    height / 2.0 - sea_depth,
                    size * (((y + 0.5) / n) - 0.5),
                ),
                ..default()
            });
        }
    }
}
