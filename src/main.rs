mod cameras;
mod debug;
mod lights;
mod terrain;
use bevy::prelude::App;
use bevy::DefaultPlugins;

use crate::cameras::CameraPlugin;
use crate::debug::DebugPlugin;
use crate::lights::LightsPlugin;
use crate::terrain::TerrainPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            LightsPlugin,
            TerrainPlugin,
            CameraPlugin,
            DebugPlugin,
        ))
        .run();
}
