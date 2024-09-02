use std::collections::HashSet;

use bevy::input::ButtonInput;
use bevy::prelude::{info, EventWriter, MouseButton, Res, ResMut, Resource};
use bevy_egui::EguiContexts;
use shared_domain::cargo_map::WithCargo;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::transport::movement_orders::{
    LoadAction, MovementOrder, MovementOrderAction, MovementOrderLocation, UnloadAction,
};
use shared_domain::TransportId;

use crate::cameras::CameraControlEvent;
use crate::communication::domain::ClientMessageEvent;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::hud::domain::{SelectType, SelectedMode};
use crate::hud::player_layout_job;
use crate::on_ui;
use crate::selection::HoveredTile;

#[derive(Resource, Default)]
pub struct TransportsToShow(HashSet<TransportId>);

impl TransportsToShow {
    #[must_use]
    pub fn contains(&self, transport_id: TransportId) -> bool {
        self.0.contains(&transport_id)
    }

    pub fn insert(&mut self, transport_id: TransportId) {
        self.0.insert(transport_id);
    }

    pub fn remove(&mut self, transport_id: TransportId) {
        self.0.remove(&transport_id);
    }

    pub fn toggle(&mut self, transport_id: TransportId) {
        if self.contains(transport_id) {
            self.remove(transport_id);
        } else {
            self.insert(transport_id);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn select_station_to_add_to_movement_orders(
    mut egui_contexts: EguiContexts,
    selected_mode: Res<SelectedMode>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    hovered_tile: Res<HoveredTile>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    game_state_resource: Option<Res<GameStateResource>>,
) {
    // Later: When do we switch away from station appending mode?

    if on_ui(&mut egui_contexts) {
        return;
    }

    if let Some(game_state_resource) = game_state_resource {
        if mouse_buttons.just_released(MouseButton::Left) {
            if let SelectedMode::Select(
                SelectType::StationToAppendToTransportMovementInstructions(transport_id),
            ) = *selected_mode
            {
                let HoveredTile(hovered_tile) = hovered_tile.as_ref();
                if let Some(hovered_tile) = hovered_tile {
                    let GameStateResource(game_state) = game_state_resource.as_ref();

                    if let Some(transport_info) = game_state.get_transport_info(transport_id) {
                        let mut new_movement_orders = transport_info.movement_orders().clone();

                        if let Some(station) = game_state.building_state().station_at(*hovered_tile)
                        {
                            let station_id = station.id();

                            let movement_order = MovementOrder {
                                go_to:  MovementOrderLocation::Station(station_id),
                                action: MovementOrderAction::UnloadAndLoad(
                                    UnloadAction::Unload,
                                    LoadAction::Load,
                                ),
                            };

                            new_movement_orders.push(movement_order);

                            client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                                game_state.game_id(),
                                GameCommand::UpdateTransportMovementOrders(
                                    transport_id,
                                    new_movement_orders,
                                ),
                            )));
                        } else {
                            info!("No station found at hovered tile {:?}", hovered_tile,);
                        }
                    } else {
                        info!("Transport {:?} not found in game state", transport_id,);
                    }
                }
            }
        }
    }
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_lines,
    clippy::unwrap_used
)]
pub(crate) fn show_transport_details(
    mut contexts: EguiContexts,
    game_state_resource: Option<Res<GameStateResource>>,
    mut show_transport_details: ResMut<TransportsToShow>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    mut selected_mode: ResMut<SelectedMode>,
    mut camera_control_events: EventWriter<CameraControlEvent>,
    player_id_resource: Res<PlayerIdResource>,
) {
    if let Some(game_state_resource) = game_state_resource {
        let GameStateResource(game_state) = game_state_resource.as_ref();
        let transports = game_state.transport_infos();
        let PlayerIdResource(player_id) = player_id_resource.as_ref();

        for transport in transports {
            if show_transport_details.contains(transport.transport_id()) {
                egui::Window::new(format!("Transport {:?}", transport.transport_id())).show(
                    contexts.ctx_mut(),
                    |ui| {
                        let movement_orders = transport.movement_orders();

                        // Later: More properly use the Window::open() method for a close button in the title bar
                        if ui.button("Close").clicked() {
                            show_transport_details.remove(transport.transport_id());
                        }
                        egui::Grid::new("transport_details")
                            .num_columns(2)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Transport ID");
                                ui.label(format!("{:?}", transport.transport_id()));
                                ui.end_row();
                                ui.label("Owner");
                                ui.label(player_layout_job(*player_id, game_state.players().get(transport.owner_id()).unwrap()));
                                ui.end_row();
                                ui.label("Transport Type");
                                ui.label(format!("{:?}", transport.transport_type()));
                                ui.end_row();
                                ui.label("Location");
                                if ui.button(format!("🔍 {:?}", transport.location())).clicked() {
                                    camera_control_events.send(CameraControlEvent::FocusOnTile(transport.location().next_tile_in_path().tile_coords_xz));
                                }
                                ui.end_row();
                                ui.label("Velocity");
                                ui.label(format!("{:?}", transport.velocity()));
                                ui.end_row();
                                ui.label("Cargo Loaded");
                                ui.label(transport.cargo_as_string());
                                ui.end_row();
                                ui.label("Cargo Processing");
                                ui.label(format!("{:?}", transport.cargo_processing()));
                                ui.end_row();
                                ui.label("Force Stopped");
                                if ui.button(format!("{:?}", movement_orders.is_force_stopped())).clicked() {
                                    let mut new_movement_orders = movement_orders.clone();
                                    new_movement_orders.set_force_stop(!movement_orders.is_force_stopped());
                                    client_messages.send(ClientMessageEvent::new(
                                        ClientCommand::Game(
                                            game_state.game_id(),
                                            GameCommand::UpdateTransportMovementOrders(
                                                transport.transport_id(),
                                                new_movement_orders,
                                            ),
                                        ),
                                    ));
                                };
                                ui.end_row();
                            });
                        egui::Grid::new("transport_movement_orders")
                            .num_columns(5)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Index");
                                ui.label("Go To");
                                ui.label("Action");
                                ui.label("Cargo");
                                ui.label("Location");
                                ui.label("");
                                ui.end_row();
                                for (idx, movement_order) in movement_orders.into_iter().enumerate() {
                                    let MovementOrderLocation::Station(station_id) = movement_order.go_to;
                                    let station = game_state.building_state().find_station(station_id).unwrap();
                                    let reference_tile = station.reference_tile();

                                    let current_order = if idx == movement_orders.next_index() {
                                        "➡ "
                                    } else {
                                        "  "
                                    };
                                    ui.label(format!("{current_order} {idx}"));
                                    ui.label(format!("{:?}", movement_order.go_to));
                                    ui.label(format!("{:?}", movement_order.action));
                                    ui.label(format!("{:?}", station.cargo()));

                                    if ui.button(format!("🔍 {reference_tile:?}")).clicked() {
                                        camera_control_events.send(CameraControlEvent::FocusOnTile(reference_tile));
                                    }

                                    // Later: Remove is disabled if there is only one movement order, as you cannot remove the last one
                                    if ui.button("❎ Remove").clicked() {
                                        info!(
                                            "Transport {:?}: Removing movement order {idx:?}",
                                            transport.transport_id()
                                        );
                                        let mut new_movement_orders = movement_orders.clone();
                                        new_movement_orders.remove_by_index(idx);
                                        client_messages.send(ClientMessageEvent::new(
                                            ClientCommand::Game(
                                                game_state.game_id(),
                                                GameCommand::UpdateTransportMovementOrders(
                                                    transport.transport_id(),
                                                    new_movement_orders,
                                                ),
                                            ),
                                        ));
                                    };
                                    ui.end_row();
                                }
                                if ui.button("➕ Add").clicked() {
                                    info!(
                                        "Transport {:?}: Switching to station selection in order to add to movement orders",
                                        transport.transport_id(),
                                    );
                                    *selected_mode = SelectedMode::Select(SelectType::StationToAppendToTransportMovementInstructions(transport.transport_id()));
                                };
                                ui.end_row();
                            });
                    },
                );
            }
        }
    }
}
