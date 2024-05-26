use std::f32::consts::FRAC_PI_2;

use bevy::app::App;
use bevy::asset::Assets;
use bevy::core::Name;
use bevy::pbr::{AlphaMode, PbrBundle, StandardMaterial};
use bevy::prelude::{
    default, Color, Commands, Mesh, Plugin, Rectangle, Res, ResMut, Startup, Transform,
};

use crate::level::domain::Level;
use crate::level::terrain::util::mesh_from_height_map_data;

mod util;

pub(crate) struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_water);
        app.add_systems(Startup, create_land);
    }
}

#[allow(
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value,
    clippy::cast_lossless
)]
fn create_water(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level: Res<Level>,
) {
    let rectangle = Rectangle::new(level.terrain.size_x as f32, level.terrain.size_z as f32);
    let mesh = meshes.add(rectangle);

    let (above, below) = &level.water.between;
    let water_level = (above.0 as f32 + below.0 as f32) / 2.0;
    let mut transform = Transform::from_xyz(0.0, water_level, 0.0);
    transform.rotate_x(-FRAC_PI_2);

    commands.spawn((
        PbrBundle {
            mesh,
            material: materials.add(StandardMaterial {
                base_color: Color::rgba_u8(173, 216, 230, 96),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            transform,
            ..default()
        },
        Name::new("Water"),
    ));
}

#[allow(
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value,
    clippy::cast_lossless
)]
fn create_land(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level: Res<Level>,
) {
    let data_slice: Vec<Vec<f32>> = level
        .terrain
        .height_map
        .iter()
        .map(|row| row.iter().map(|h| h.0 as f32).collect::<Vec<_>>())
        .collect();
    let half_x = (level.terrain.size_x as f32) / 2.0;
    let half_z = (level.terrain.size_z as f32) / 2.0;
    let mesh = mesh_from_height_map_data(-half_x, half_x, -half_z, half_z, data_slice);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::DARK_GREEN),
            transform: Transform::default(),
            ..default()
        },
        Name::new("Land"),
    ));
}
