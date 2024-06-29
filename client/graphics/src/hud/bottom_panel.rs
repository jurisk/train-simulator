use bevy::prelude::Res;
use bevy_egui::EguiContexts;
use egui::{Direction, Layout};

use crate::hud::domain::SelectedMode;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn show_bottom_panel(mut contexts: EguiContexts, selected_mode: Res<SelectedMode>) {
    egui::TopBottomPanel::bottom("hud_bottom_panel").show(contexts.ctx_mut(), |ui| {
        ui.with_layout(
            Layout::centered_and_justified(Direction::LeftToRight),
            |ui| {
                ui.label(format!("Selected mode: {:?}", *selected_mode));
            },
        );
    });
}
