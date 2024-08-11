use bevy::prelude::{Res, ResMut};
use bevy::utils::default;
use bevy_egui::EguiContexts;
use egui::text::LayoutJob;
use egui::{Color32, TextFormat, Ui};
use shared_domain::building::building_info::WithOwner;
use shared_domain::building::building_state::BuildingState;
use shared_domain::players::player_state::PlayerState;
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::PlayerId;

use crate::game::transport::ui::TransportsToShow;
use crate::game::{GameStateResource, PlayerIdResource};

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn show_left_panel(
    mut contexts: EguiContexts,
    game_state_resource: Option<Res<GameStateResource>>,
    mut transport_to_show: ResMut<TransportsToShow>,
    player_id_resource: Option<Res<PlayerIdResource>>,
) {
    if let Some(player_id_resource) = player_id_resource {
        let PlayerIdResource(player_id) = player_id_resource.as_ref();
        if let Some(game_state_resource) = game_state_resource {
            let GameStateResource(game_state) = game_state_resource.as_ref();

            egui::SidePanel::left("hud_left_panel").show(contexts.ctx_mut(), |ui| {
                players_info_panel(ui, game_state.players());
                buildings_info_panel(ui, *player_id, game_state.building_state());
                transport_info_panel(
                    ui,
                    *player_id,
                    game_state.transport_infos(),
                    &mut transport_to_show,
                );
            });
        }
    }
}

#[allow(clippy::match_same_arms)]
fn buildings_info_panel(ui: &mut Ui, player_id: PlayerId, buildings: &BuildingState) {
    ui.heading("Industry");
    for building in buildings.all_industry_buildings() {
        if building.owner_id() == player_id {
            ui.label(format!("{building:?}"));
        }
    }
    ui.heading("Stations");
    for building in buildings.all_stations() {
        if building.owner_id() == player_id {
            ui.label(format!("{building:?}"));
        }
    }
}

fn transport_info_panel(
    ui: &mut Ui,
    player_id: PlayerId,
    transport_infos: &Vec<TransportInfo>,
    transports_to_show: &mut ResMut<TransportsToShow>,
) {
    ui.heading("Transports");
    for transport_info in transport_infos {
        if transport_info.owner_id() == player_id {
            let id = transport_info.transport_id();
            let selected = transports_to_show.contains(id);

            if ui
                .add(egui::Button::new(format!("{transport_info:?}")).selected(selected))
                .clicked()
            {
                if selected {
                    transports_to_show.remove(id);
                } else {
                    transports_to_show.insert(id);
                }
            }
        }
    }
}

#[allow(clippy::similar_names)]
fn players_info_panel(ui: &mut Ui, players: &PlayerState) {
    ui.heading("Players");
    for player_info in players.infos() {
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
