#![allow(clippy::needless_pass_by_value)]

use bevy::app::{App, Update};
use bevy::input::ButtonInput;
use bevy::prelude::{KeyCode, Local, Plugin, Res, ResMut};
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_renet::renet::RenetClient;
use renet_visualizer::{RenetClientVisualizer, RenetVisualizerStyle};

pub struct MultiplayerRenetClientVisualisationPlugin;

impl Plugin for MultiplayerRenetClientVisualisationPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.insert_resource(RenetClientVisualizer::<200>::new(
            RenetVisualizerStyle::default(),
        ));
        app.add_systems(Update, update_visualizer_system);
    }
}

fn update_visualizer_system(
    mut egui_contexts: EguiContexts,
    mut visualizer: ResMut<RenetClientVisualizer<200>>,
    client: Res<RenetClient>,
    mut show_visualizer: Local<bool>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    visualizer.add_network_info(client.network_info());
    if keyboard_input.just_pressed(KeyCode::F1) {
        *show_visualizer = !*show_visualizer;
    }
    if *show_visualizer {
        visualizer.show_window(egui_contexts.ctx_mut());
    }
}
