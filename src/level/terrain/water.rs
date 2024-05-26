use std::f32::consts::FRAC_PI_2;

use bevy::app::{App, Plugin};
use bevy::core::Name;
use bevy::prelude::{
    default, AlphaMode, Assets, Color, Commands, Mesh, OnEnter, PbrBundle, Rectangle, Res, ResMut,
    StandardMaterial, Transform,
};

use crate::level::terrain::Y_COEF;
use crate::level::LevelResource;
use crate::states::GameState;

pub(crate) struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), create_water);
        // Eventually, clean-up will be also needed
    }
}

#[allow(
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value,
    clippy::cast_lossless
)]
pub(crate) fn create_water(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level_resource: Res<LevelResource>,
) {
    let level = &level_resource.level;
    let rectangle = Rectangle::new(level.terrain.size_x as f32, level.terrain.size_z as f32);
    let mesh = meshes.add(rectangle);

    let (above, below) = &level.water.between;
    let water_level = ((above.0 as f32 + below.0 as f32) / 2.0) * Y_COEF;
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
