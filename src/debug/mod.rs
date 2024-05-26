use bevy::app::{App, Update};
use bevy::math::Vec3;
use bevy::prelude::{in_state, Color, Gizmos, IntoSystemConfigs, Plugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::states::GameState;

pub(crate) struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldInspectorPlugin::new())
            .add_systems(Update, draw_test_axis.run_if(in_state(GameState::Playing)));
    }
}

fn draw_test_axis(mut gizmos: Gizmos) {
    let length = 2.0;
    gizmos.arrow(Vec3::ZERO, Vec3::new(length, 0.0, 0.0), Color::RED);
    gizmos.arrow(Vec3::ZERO, Vec3::new(0.0, length, 0.0), Color::GREEN);
    gizmos.arrow(Vec3::ZERO, Vec3::new(0.0, 0.0, length), Color::BLUE);
}
