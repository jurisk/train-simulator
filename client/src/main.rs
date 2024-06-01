use bevy::asset::AssetPlugin;
use bevy::prelude::PluginGroup;
use bevy::prelude::{App, AssetMode};
use bevy::utils::default;
use bevy::window::{Window, WindowPlugin, WindowResolution};
use bevy::DefaultPlugins;

use crate::cameras::CameraPlugin;
use crate::communication::CommunicationPlugin;
use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::debug::DebugPlugin;
use crate::level::LevelPlugin;
use crate::lights::LightsPlugin;
use crate::states::GameState;

mod cameras;
mod communication;
mod constants;
mod debug;
mod level;
mod lights;
mod states;

fn main() {
    let mut app = App::new();
    app.init_state::<GameState>();
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
                mode: AssetMode::Unprocessed, // Processed requires for the asset processor features to be enabled
                ..default()
            }),
    );
    app.add_plugins((
        CommunicationPlugin,
        LightsPlugin,
        LevelPlugin,
        CameraPlugin,
        DebugPlugin,
    ));
    app.run();
}
