use bevy::prelude::{Res, ResMut};
use bevy_egui::EguiContexts;
use egui::{Align, Layout};

use crate::hud::PointerOverHud;
use crate::hud::domain::SelectedMode;
use crate::selection::{ClickedEdge, ClickedTile, HoveredEdge, HoveredTile};

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn show_bottom_panel(
    mut contexts: EguiContexts,
    selected_mode: Res<SelectedMode>,
    clicked_tile: Res<ClickedTile>,
    clicked_edge: Res<ClickedEdge>,
    hovered_tile: Res<HoveredTile>,
    hovered_edge: Res<HoveredEdge>,
    mut pointer_over_hud: ResMut<PointerOverHud>,
) {
    egui::TopBottomPanel::bottom("hud_bottom_panel").show(contexts.ctx_mut(), |ui| {
        pointer_over_hud.apply(ui);

        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.label(format!("Selected mode: {:?}", selected_mode.as_ref()));
            ui.label(format!("{clicked_tile:?}"));
            ui.label(format!("{clicked_edge:?}"));
            ui.label(format!("{hovered_tile:?}"));
            ui.label(format!("{hovered_edge:?}"));
            ui.label(
                if pointer_over_hud.get() {
                    "Over HUD"
                } else {
                    "Not over HUD"
                },
            );
        });
    });
}
