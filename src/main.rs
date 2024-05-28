mod cameras;
mod communication;
mod debug;
mod level;
mod lights;
mod states;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::App;
use bevy::DefaultPlugins;

use crate::cameras::CameraPlugin;
use crate::communication::CommunicationPlugin;
use crate::debug::DebugPlugin;
use crate::level::LevelPlugin;
use crate::lights::LightsPlugin;
use crate::states::GameState;

fn main() {
    App::new()
        .init_state::<GameState>()
        .add_plugins((
            DefaultPlugins,
            CommunicationPlugin,
            LightsPlugin,
            LevelPlugin,
            CameraPlugin,
            DebugPlugin,
        ))
        .insert_resource(AssetMetaCheck::Never) // Otherwise we were getting 404-s in WASM for `*.wgsl.meta` files
        .run();
}
