use std::collections::HashSet;

use bevy::prelude::{info, EventWriter, Res, ResMut, Resource};
use bevy_egui::EguiContexts;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::TransportId;

use crate::communication::domain::ClientMessageEvent;
use crate::game::GameStateResource;

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
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn show_transport_details(
    mut contexts: EguiContexts,
    game_state_resource: Option<Res<GameStateResource>>,
    mut show_transport_details: ResMut<TransportsToShow>,
    mut client_messages: EventWriter<ClientMessageEvent>,
) {
    if let Some(game_state_resource) = game_state_resource {
        let GameStateResource(game_state) = game_state_resource.as_ref();
        let transports = game_state.transport_infos();

        for transport in transports {
            if show_transport_details.contains(transport.transport_id()) {
                egui::Window::new(format!("Transport {:?}", transport.transport_id())).show(
                    contexts.ctx_mut(),
                    |ui| {
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
                                ui.label("Owner ID");
                                ui.label(format!("{:?}", transport.owner_id()));
                                ui.end_row();
                                ui.label("Transport Type");
                                ui.label(format!("{:?}", transport.transport_type()));
                                ui.end_row();
                                ui.label("Location");
                                ui.label(format!("{:?}", transport.location()));
                                ui.end_row();
                                ui.label("Velocity");
                                ui.label(format!("{:?}", transport.velocity()));
                                ui.end_row();
                                ui.label("Cargo Loaded");
                                ui.label(format!("{:?}", transport.cargo_loaded()));
                                ui.end_row();
                                ui.label("Cargo Processing");
                                ui.label(format!("{:?}", transport.cargo_processing()));
                                ui.end_row();
                            });
                        let movement_orders = transport.movement_orders();
                        egui::Grid::new("transport_movement_orders")
                            .num_columns(4)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Force Stopped");
                                ui.label(format!("{:?}", movement_orders.is_force_stopped()));
                                ui.end_row();
                                for (idx, movement_order) in movement_orders.into_iter().enumerate()
                                {
                                    let current_order = if idx == movement_orders.next_index() {
                                        "➡ "
                                    } else {
                                        "  "
                                    };
                                    ui.label(format!("{current_order} {idx}"));
                                    ui.label(format!("{:?}", movement_order.go_to));
                                    ui.label(format!("{:?}", movement_order.action));

                                    // Later: Remove is disabled if there is only one movement order, as you cannot remove the last one
                                    if ui.button("Remove").clicked() {
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
                                if ui.button("Add").clicked() {
                                    // TODO HIGH: Allow selecting a station and Add movement order
                                    info!(
                                        "Transport {:?}: Adding movement order",
                                        transport.transport_id()
                                    );
                                };
                                ui.end_row();
                            });
                    },
                );
            }
        }
    }
}
