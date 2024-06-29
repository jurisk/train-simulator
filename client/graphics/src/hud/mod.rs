use bevy::app::{App, Plugin};
use bevy::prelude::Update;
use bevy_egui::EguiPlugin;

use crate::hud::domain::SelectedMode;
use crate::hud::left_panel::LeftPanel;
use crate::hud::mode_selection::show_mode_selection_hud;

pub mod domain;
pub mod left_panel;
pub mod mode_selection;

pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.insert_resource(SelectedMode::Info);
        app.add_plugins(LeftPanel);
        app.add_systems(Update, show_mode_selection_hud);
    }
}
