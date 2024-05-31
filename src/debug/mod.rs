use bevy::app::{App, Update};
use bevy::diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::math::Vec3;
use bevy::prelude::{
    default, in_state, Color, Commands, Gizmos, IntoSystemConfigs, Name, Plugin, Query, Res,
    Startup, Text, TextBundle, TextStyle,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::states::GameState;

pub(crate) struct DebugPlugin;

// This has some weird race conditions, and it doesn't always appear, but I hope when
// https://github.com/bevyengine/bevy/blob/main/crates/bevy_dev_tools/src/fps_overlay.rs gets
// released, we can migrate to that and it will work.
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldInspectorPlugin::new())
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Update, draw_test_axis.run_if(in_state(GameState::Playing)))
            .add_systems(Startup, setup_fps)
            .add_systems(Update, update_fps);
    }
}

fn draw_test_axis(mut gizmos: Gizmos) {
    let length = 2.0;
    gizmos.arrow(Vec3::ZERO, Vec3::new(length, 0.0, 0.0), Color::RED);
    gizmos.arrow(Vec3::ZERO, Vec3::new(0.0, length, 0.0), Color::GREEN);
    gizmos.arrow(Vec3::ZERO, Vec3::new(0.0, 0.0, length), Color::BLUE);
}

fn setup_fps(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "FPS: ???\n\n",
            TextStyle {
                font_size: 30.,
                ..default()
            },
        ),
        Name::new("FPS info"),
    ));
}

#[allow(clippy::needless_pass_by_value)]
fn update_fps(mut query: Query<&mut Text>, diag: Res<DiagnosticsStore>) {
    let mut text = query.single_mut();

    if let Some(fps) = diag
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::smoothed)
    {
        text.sections[0].value = format!("FPS: {fps:.0}\n\n");
    };
}
