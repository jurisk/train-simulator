use bevy::app::App;
use bevy::math::Vec3;
use bevy::prelude::{default, Camera3dBundle, Commands, Plugin, Startup, Transform};

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_cameras);
    }
}

fn create_cameras(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(2.5, 12.0, 2.5).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
