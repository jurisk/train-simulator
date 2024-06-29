use std::collections::HashMap;

use bevy::prelude::{App, Plugin, Res, Update};
use bevy::utils::default;
use bevy_egui::EguiContexts;
use egui::text::LayoutJob;
use egui::{Color32, TextFormat, Ui};
use shared_domain::server_response::PlayerInfo;
use shared_domain::PlayerId;

use crate::game::GameStateResource;

pub(crate) struct LeftPanel;

impl Plugin for LeftPanel {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, left_panel);
    }
}

pub(crate) fn left_panel(
    mut contexts: EguiContexts,
    game_state_resource: Option<Res<GameStateResource>>,
) {
    if let Some(game_state_resource) = game_state_resource {
        let GameStateResource(game_state) = game_state_resource.as_ref();

        egui::SidePanel::left("hud_left_panel")
            .show(contexts.ctx_mut(), |ui| {
                players_info_panel(ui, game_state.players());
            });
    }
}

#[allow(clippy::similar_names)]
fn players_info_panel(ui: &mut Ui, players: &HashMap<PlayerId, PlayerInfo>) {
    ui.heading("Players");
    for player_info in players.values() {
        let colour = player_info.colour;
        let color = Color32::from_rgb(colour.r, colour.g, colour.b);

        let mut job = LayoutJob::default();
        job.append("⬛", 0.0, TextFormat { color, ..default() });

        job.append(
            format!("{}", player_info.name).as_str(),
            0.0,
            TextFormat { ..default() },
        );

        ui.label(job);
    }
}
