use bevy::app::{App, Plugin};
use bevy::prelude::Update;

use crate::hud::domain::SelectedMode;
use crate::hud::mode_selection::show_mode_selection_hud;

pub mod domain;
pub mod mode_selection;

pub(crate) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, show_mode_selection_hud);
        app.insert_resource(SelectedMode::Info);
    }
}
