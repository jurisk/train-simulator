#![feature(let_chains)]

use bevy::asset::AssetPlugin;
use bevy::prelude::{App, AssetMode, Plugin, PluginGroup};
use bevy::utils::default;
use bevy::window::{PresentMode, Window, WindowPlugin, WindowResolution};
use bevy::DefaultPlugins;

use crate::cameras::CameraPlugin;
use crate::communication::CommunicationPlugin;
use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::debug::DebugPlugin;
use crate::game::GamePlugin;
use crate::lights::LightsPlugin;
use crate::lobby::LobbyHandlerPlugin;
use crate::selection::SelectionPlugin;

mod cameras;
pub mod communication;
mod constants;
mod debug;
mod game;
mod lights;
mod lobby;
mod selection;
pub mod states;

pub struct ClientGraphicsPlugin;

impl Plugin for ClientGraphicsPlugin {
    fn build(&self, app: &mut App) {
        let asset_path_prefix = if cfg!(target_arch = "wasm32") {
            ""
        } else {
            "../../client/graphics/"
        };

        app.add_plugins(
            DefaultPlugins
                .build()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        #[allow(clippy::cast_precision_loss)]
                        resolution: WindowResolution::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
                        present_mode: PresentMode::AutoNoVsync, // For bevy_mod_raycast, see low_latency_window_plugin()
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: (asset_path_prefix.to_owned() + "assets").to_string(),
                    processed_file_path: (asset_path_prefix.to_owned() + "processed-assets").to_string(),
                    mode: AssetMode::Unprocessed, // Processed requires for the asset processor features to be enabled
                    ..default()
                }),
        );
        app.add_plugins((
            CommunicationPlugin,
            LightsPlugin,
            LobbyHandlerPlugin,
            GamePlugin,
            CameraPlugin,
            DebugPlugin,
            SelectionPlugin,
        ));
    }
}
