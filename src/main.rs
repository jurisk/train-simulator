mod cameras;
mod debug;
mod world;

use crate::cameras::CameraPlugin;
use crate::debug::DebugPlugin;
use crate::world::WorldPlugin;
use bevy::prelude::App;
use bevy::DefaultPlugins;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WorldPlugin, CameraPlugin, DebugPlugin))
        .run();
}
