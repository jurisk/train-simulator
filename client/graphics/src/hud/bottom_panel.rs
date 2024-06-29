use bevy_egui::EguiContexts;
use egui::{Direction, Layout};

pub(crate) fn show_bottom_panel(mut contexts: EguiContexts) {
    egui::TopBottomPanel::bottom("hud_bottom_panel").show(contexts.ctx_mut(), |ui| {
        ui.with_layout(
            Layout::centered_and_justified(Direction::LeftToRight),
            |ui| {
                ui.label("Bottom panel");
            },
        );
    });
}
