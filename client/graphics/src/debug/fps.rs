use bevy::app::{App, Update};
use bevy::diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::{
    default, Commands, Name, Plugin, Query, Res, Startup, Text, TextBundle, TextStyle,
};

// This has some weird race conditions, and it doesn't always appear, but I hope when
// https://github.com/bevyengine/bevy/blob/main/crates/bevy_dev_tools/src/fps_overlay.rs gets
// released, we can migrate to that and it will work better. Or we need to have "assets loaded" state.
// https://github.com/IyesGames/iyes_perf_ui also exists.
pub(crate) struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, setup_fps)
            .add_systems(Update, update_fps);
    }
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
