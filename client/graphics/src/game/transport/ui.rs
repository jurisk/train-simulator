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
                        // TODO HIGH: Allow adjusting MovementOrders
                        ui.label(format!("{transport:?}"));
                    },
                );
            }
        }
    }
}
