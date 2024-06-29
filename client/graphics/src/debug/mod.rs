use std::time::Duration;

use bevy::app::App;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::input::ButtonInput;
use bevy::prelude::{KeyCode, Plugin, Res, ResMut, Resource, Update};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::debug::fps::FpsPlugin;
use crate::debug::test_axis::TestAxisPlugin;
use crate::key_map;

mod fps;
mod test_axis;

#[derive(Resource)]
struct ShowWorldInspector(bool);

impl ShowWorldInspector {
    fn toggle(&mut self) {
        self.0 = !self.0;
    }
}

pub(crate) struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShowWorldInspector(false))
            .add_plugins(WorldInspectorPlugin::new().run_if(show_world_inspector_window))
            .add_systems(Update, show_world_inspector_keyboard)
            .add_plugins(FpsPlugin)
            .add_plugins(TestAxisPlugin)
            .add_plugins(LogDiagnosticsPlugin {
                debug:         false,
                wait_duration: Duration::from_secs(60),
                filter:        None,
            });
    }
}

#[allow(clippy::needless_pass_by_value)]
fn show_world_inspector_window(show_world_inspector: Res<ShowWorldInspector>) -> bool {
    let ShowWorldInspector(show_world_inspector) = show_world_inspector.as_ref();
    *show_world_inspector
}

#[allow(clippy::needless_pass_by_value)]
fn show_world_inspector_keyboard(
    mut show_world_inspector: ResMut<ShowWorldInspector>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(key_map::TOGGLE_WORLD_INSPECTOR) {
        show_world_inspector.as_mut().toggle();
    }
}
