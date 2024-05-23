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
        app.add_systems(Startup, create_water);
        app.add_systems(Startup, create_land);
    }
}

const SIZE: f32 = 10.0;
const SEA_DEPTH: f32 = 2.5;

fn create_water(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    #[allow(deprecated)]
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane::from_size(SIZE)),
        material: materials.add(StandardMaterial {
            base_color: Color::rgba_u8(173, 216, 230, 96),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
}

fn create_land(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();
    let n = 6u8;
    for x in 0 .. n {
        for y in 0 .. n {
            let x = f32::from(x);
            let y = f32::from(y);
            let n = f32::from(n);
            let height = f32::from(rng.gen_range(1u8 ..= 5u8));
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(SIZE / n, height, SIZE / n)),
                material: materials.add(Color::rgb(rng.gen(), rng.gen(), rng.gen())),
                transform: Transform::from_xyz(
                    SIZE * (((x + 0.5) / n) - 0.5),
                    height / 2.0 - SEA_DEPTH,
                    SIZE * (((y + 0.5) / n) - 0.5),
                ),
                ..default()
            });
        }
    }
}
