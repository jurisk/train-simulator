use std::collections::HashSet;

use bevy::prelude::{Res, Resource};
use bevy_egui::EguiContexts;
use shared_domain::TransportId;

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
    show_transport_details: Res<TransportsToShow>,
) {
    if let Some(game_state_resource) = game_state_resource {
        let GameStateResource(game_state) = game_state_resource.as_ref();
        let transports = game_state.transport_infos();
        let TransportsToShow(transports_to_show) = show_transport_details.as_ref();

        for transport in transports {
            if transports_to_show.contains(&transport.transport_id()) {
                egui::Window::new(format!("Transport {:?}", transport.transport_id())).show(
                    contexts.ctx_mut(),
                    |ui| {
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
                            .num_columns(2)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Force Stopped");
                                ui.label(format!("{:?}", movement_orders.is_force_stopped()));
                                ui.end_row();
                                for movement_order in movement_orders {
                                    ui.label(format!("{:?}", movement_order.go_to));
                                    ui.label(format!("{:?}", movement_order.action));
                                    ui.end_row();
                                }
                            });
                    },
                );
            }
        }
    }
}
