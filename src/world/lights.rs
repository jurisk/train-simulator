use bevy::app::{App, Startup};
use bevy::pbr::{PointLight, PointLightBundle};
use bevy::prelude::{default, Commands, Plugin, Transform};

pub(crate) struct LightsPlugin;

impl Plugin for LightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_lights);
    }
}

fn create_lights(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}
