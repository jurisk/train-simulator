use bevy::app::{App, Update};
use bevy::prelude::{EventReader, Plugin, Res, ResMut};
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_renet::renet::{RenetServer, ServerEvent};
use renet_visualizer::RenetServerVisualizer;

pub struct MultiplayerRenetServerVisualisationPlugin;

impl Plugin for MultiplayerRenetServerVisualisationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin);

        app.insert_resource(RenetServerVisualizer::<200>::default());
        app.add_systems(Update, update_visualizer_system);
        app.add_systems(Update, handle_events_system);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn update_visualizer_system(
    mut egui_contexts: EguiContexts,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    server: Res<RenetServer>,
) {
    visualizer.update(&server);
    visualizer.show_window(egui_contexts.ctx_mut());
}

fn handle_events_system(
    mut server_events: EventReader<ServerEvent>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                visualizer.add_client(*client_id);
            },
            ServerEvent::ClientDisconnected {
                client_id,
                reason: _,
            } => {
                visualizer.remove_client(*client_id);
            },
        }
    }
}
