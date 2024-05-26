use bevy::asset::Assets;
use bevy::core::Name;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{default, Color, Commands, Mesh, Res, ResMut, Transform};

use crate::level::terrain::util::mesh_from_height_map_data;
use crate::level::terrain::Y_COEF;
use crate::level::LevelResource;

#[allow(
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value,
    clippy::cast_lossless
)]
pub(crate) fn create_land(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level_resource: Res<LevelResource>,
) {
    let level = &level_resource.level;
    let data_slice: Vec<Vec<f32>> = level
        .terrain
        .height_map
        .iter()
        .map(|row| row.iter().map(|h| h.0 as f32).collect::<Vec<_>>())
        .collect();
    let half_x = (level.terrain.size_x as f32) / 2.0;
    let half_z = (level.terrain.size_z as f32) / 2.0;
    let mesh = mesh_from_height_map_data(-half_x, half_x, -half_z, half_z, Y_COEF, data_slice);
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
