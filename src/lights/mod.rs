use std::f32::consts::PI;

use bevy::app::{App, Startup};
use bevy::core::Name;
use bevy::pbr::{DirectionalLight, DirectionalLightBundle};
use bevy::prelude::light_consts::lux::CLEAR_SUNRISE;
use bevy::prelude::{default, Commands, Plugin, Quat, Transform, Vec3};

pub(crate) struct LightsPlugin;

impl Plugin for LightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_lights);
    }
}

fn create_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: CLEAR_SUNRISE,
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
