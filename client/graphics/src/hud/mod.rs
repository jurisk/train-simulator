use bevy::app::{App, Plugin, Update};
use bevy::prelude::IntoSystemConfigs;
use bevy_egui::EguiPlugin;

use crate::hud::domain::SelectedMode;
use crate::hud::labels::draw_labels;

pub mod bottom_panel;
pub mod domain;
pub mod labels;
pub mod left_panel;
pub mod top_panel;

pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.insert_resource(SelectedMode::Info);

        app.add_systems(Update, bottom_panel::show_bottom_panel);
        app.add_systems(
            Update,
            top_panel::show_top_panel.after(bottom_panel::show_bottom_panel),
        );
        app.add_systems(
            Update,
            left_panel::show_left_panel.after(top_panel::show_top_panel),
        );
        app.add_systems(Update, draw_labels.after(left_panel::show_left_panel));
    }
}
