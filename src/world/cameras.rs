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
    let n = 8.0;
    let from = Transform::from_xyz(n, n, n);
    let target = Vec3::ZERO;
    let up = Vec3::Y;

    commands.spawn(Camera3dBundle {
        transform: from.looking_at(target, up),
        ..default()
    });
}
