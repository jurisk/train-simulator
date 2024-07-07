use bevy::app::{App, Update};
use bevy::color::palettes::css::{BLUE, GREEN, RED};
use bevy::math::Vec3;
use bevy::prelude::{in_state, Gizmos, IntoSystemConfigs, Plugin};

use crate::states::ClientState;

pub(crate) struct TestAxisPlugin;

impl Plugin for TestAxisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            draw_test_axis.run_if(in_state(ClientState::Playing)),
        );
    }
}

fn draw_test_axis(mut gizmos: Gizmos) {
    let length = 2.0;
    gizmos.arrow(Vec3::ZERO, Vec3::new(length, 0.0, 0.0), RED);
    gizmos.arrow(Vec3::ZERO, Vec3::new(0.0, length, 0.0), GREEN);
    gizmos.arrow(Vec3::ZERO, Vec3::new(0.0, 0.0, length), BLUE);
}
