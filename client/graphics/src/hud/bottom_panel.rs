use bevy::prelude::Res;
use bevy_egui::EguiContexts;
use egui::{Align, Layout};

use crate::hud::domain::SelectedMode;
use crate::selection::HoveredTile;

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn show_bottom_panel(
    mut contexts: EguiContexts,
    selected_mode: Res<SelectedMode>,
    hovered_tile: Res<HoveredTile>,
) {
    egui::TopBottomPanel::bottom("hud_bottom_panel").show(contexts.ctx_mut(), |ui| {
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.label(format!("{hovered_tile:?}"));
            ui.label(format!("Selected mode: {:?}", selected_mode.as_ref()));
        });
    });
}
