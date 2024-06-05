use std::time::Duration;

use bevy::app::App;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::Plugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::debug::fps::FpsPlugin;
use crate::debug::test_axis::TestAxisPlugin;

mod fps;
mod test_axis;

pub(crate) struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldInspectorPlugin::new())
            .add_plugins(FpsPlugin)
            .add_plugins(TestAxisPlugin)
            .add_plugins(LogDiagnosticsPlugin {
                debug:         false,
                wait_duration: Duration::from_secs(60),
                filter:        None,
            });
    }
}
