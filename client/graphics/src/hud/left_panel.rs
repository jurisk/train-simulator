use bevy::prelude::{info, EventWriter, Res, ResMut};
use bevy_egui::EguiContexts;
use egui::Ui;
use shared_domain::building::building_info::WithOwner;
use shared_domain::building::building_state::BuildingState;
use shared_domain::cargo_map::WithCargo;
use shared_domain::players::player_state::PlayerState;
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::PlayerId;

use crate::cameras::CameraControlEvent;
use crate::game::transport::ui::TransportsToShow;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::hud::player_layout_job;

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn show_left_panel(
    mut contexts: EguiContexts,
    game_state_resource: Option<Res<GameStateResource>>,
    mut transport_to_show: ResMut<TransportsToShow>,
    player_id_resource: Option<Res<PlayerIdResource>>,
    mut camera_control_events: EventWriter<CameraControlEvent>,
) {
    if let Some(player_id_resource) = player_id_resource {
        let PlayerIdResource(player_id) = player_id_resource.as_ref();
        if let Some(game_state_resource) = game_state_resource {
            let GameStateResource(game_state) = game_state_resource.as_ref();

            egui::SidePanel::left("hud_left_panel").show(contexts.ctx_mut(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    players_info_panel(ui, *player_id, game_state.players());
                    buildings_info_panel(
                        ui,
                        *player_id,
                        game_state.building_state(),
                        &mut camera_control_events,
                    );
                    transport_info_panel(
                        ui,
                        *player_id,
                        game_state.transport_infos(),
                        &mut transport_to_show,
                    );
                });
            });
        }
    }
}

#[expect(clippy::match_same_arms, clippy::collapsible_if)]
fn buildings_info_panel(
    ui: &mut Ui,
    player_id: PlayerId,
    buildings: &BuildingState,
    camera_control_events: &mut EventWriter<CameraControlEvent>,
) {
    ui.heading("Industry");
    for building in buildings.all_industry_buildings() {
        if building.owner_id() == player_id {
            let label = format!(
                "{:?} {:?}",
                building.reference_tile(),
                building.industry_type()
            );
            if ui.button(label).clicked() {
                camera_control_events
                    .send(CameraControlEvent::FocusOnTile(building.reference_tile()));
            }
        }
    }
    ui.heading("Stations");
    for building in buildings.all_stations() {
        if building.owner_id() == player_id {
            let label = format!("{:?} {:?}", building.reference_tile(), building.cargo());
            if ui.button(label).clicked() {
                camera_control_events
                    .send(CameraControlEvent::FocusOnTile(building.reference_tile()));
            }
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

            let label = format!("{:?} {}", id, transport_info.cargo_as_string());
            if ui
                .add(egui::Button::new(label).selected(selected))
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

#[expect(clippy::similar_names)]
fn players_info_panel(ui: &mut Ui, own_player_id: PlayerId, players: &PlayerState) {
    ui.heading("Players");
    for player_info in players.infos() {
        let job = player_layout_job(own_player_id, player_info);
        if ui.button(job).clicked() {
            // TODO: Open player panel
            info!("Player panel for player: {:?}", player_info);
        }
    }
}
