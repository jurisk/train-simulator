use bevy::asset::AssetPlugin;
use bevy::prelude::{App, AssetMode, Plugin, PluginGroup};
use bevy::utils::default;
use bevy::window::{Window, WindowPlugin, WindowResolution};
use bevy::DefaultPlugins;

use crate::cameras::CameraPlugin;
use crate::communication::CommunicationPlugin;
use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::debug::DebugPlugin;
use crate::game::GamePlugin;
use crate::lights::LightsPlugin;
use crate::states::ClientState;

mod cameras;
pub mod communication;
mod constants;
mod debug;
mod game;
mod lights;
mod states;

pub struct ClientGraphicsPlugin;

impl Plugin for ClientGraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ClientState>();
        app.add_plugins(
            DefaultPlugins
                .build()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        #[allow(clippy::cast_precision_loss)]
                        resolution: WindowResolution::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: "../../client/graphics/assets".to_string(),
                    processed_file_path: "../../client/graphics/processed-assets".to_string(),
                    mode: AssetMode::Unprocessed, // Processed requires for the asset processor features to be enabled
                    ..default()
                }),
        );
        app.add_plugins((
            CommunicationPlugin,
            LightsPlugin,
            GamePlugin,
            CameraPlugin,
            DebugPlugin,
        ));
    }
}
