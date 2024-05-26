mod cameras;
mod debug;
mod level;
mod lights;

use bevy::prelude::App;
use bevy::DefaultPlugins;

use crate::cameras::CameraPlugin;
use crate::debug::DebugPlugin;
use crate::level::LevelPlugin;
use crate::lights::LightsPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            LightsPlugin,
            LevelPlugin,
            CameraPlugin,
            DebugPlugin,
        ))
        .run();
}
