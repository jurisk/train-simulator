use std::f32::consts::PI;

use bevy::core::Name;
use bevy::pbr::{DirectionalLight, DirectionalLightBundle};
use bevy::prelude::light_consts::lux::OVERCAST_DAY;
use bevy::prelude::{
    default, App, Commands, Plugin, Quat, Query, Res, Startup, Time, Transform, Update, Vec3, With,
};

pub(crate) struct LightsPlugin;

impl Plugin for LightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_lights);
        app.add_systems(Update, animate_light_direction);
    }
}

fn create_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: OVERCAST_DAY,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_rotation_x(-PI / 4.0),
                ..default()
            },
            ..default()
        },
        Name::new("Directional Light"),
    ));
}

#[allow(clippy::needless_pass_by_value)]
fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        const LIGHT_ROTATION_COEF: f32 = 0.2;
        transform.rotate_y(time.delta_seconds() * LIGHT_ROTATION_COEF);
    }
}
