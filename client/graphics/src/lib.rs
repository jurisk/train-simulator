use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::prelude::{info, App, AssetMode, Plugin, PluginGroup};
use bevy::utils::default;
use bevy::window::{PresentMode, Window, WindowPlugin, WindowResolution};
use bevy::DefaultPlugins;

use crate::assets::GameAssetsPlugin;
use crate::cameras::CameraPlugin;
use crate::communication::CommunicationPlugin;
use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::debug::DebugPlugin;
use crate::game::{GameLaunchParams, GamePlugin};
use crate::hud::HudPlugin;
use crate::lights::LightsPlugin;
use crate::lobby::LobbyHandlerPlugin;
use crate::network::client_ping::ClientPingPlugin;
use crate::selection::SelectionPlugin;

pub mod assets;
mod cameras;
pub mod communication;
mod constants;
mod debug;
pub mod game;
pub mod hud;
pub mod key_map;
mod lights;
mod lobby;
pub mod network;
mod selection;
pub mod states;
pub mod util;

pub struct ClientGraphicsPlugin {
    pub game_launch_params: GameLaunchParams,
}

impl Plugin for ClientGraphicsPlugin {
    fn build(&self, app: &mut App) {
        let asset_path_prefix = if cfg!(target_arch = "wasm32") {
            ""
        } else {
            "../../"
        };

        app.add_plugins(
            DefaultPlugins
                .build()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        #[allow(clippy::cast_precision_loss)]
                        resolution: WindowResolution::new(
                            WINDOW_WIDTH as f32,
                            WINDOW_HEIGHT as f32,
                        ),
                        present_mode: PresentMode::AutoNoVsync, // For bevy_mod_raycast, see low_latency_window_plugin()
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: (asset_path_prefix.to_owned() + "assets").to_string(),
                    processed_file_path: (asset_path_prefix.to_owned() + "processed-assets")
                        .to_string(),
                    mode: AssetMode::Unprocessed, // Processed requires for the asset processor features to be enabled
                    ..default()
                }),
        );
        app.add_plugins((
            GameAssetsPlugin,
            CommunicationPlugin,
            LightsPlugin,
            LobbyHandlerPlugin,
            GamePlugin {
                game_launch_params: self.game_launch_params.clone(),
            },
            CameraPlugin,
            DebugPlugin,
            SelectionPlugin,
            ClientPingPlugin {
                interval: Duration::from_secs(60),
            },
            HudPlugin,
        ));

        info!("Arch: {}", std::env::consts::ARCH);
        info!("Target OS: {}", std::env::consts::OS);
        info!("Asset path prefix: {}", asset_path_prefix);
        info!("Current dir: {:?}", std::env::current_dir());
    }
}
