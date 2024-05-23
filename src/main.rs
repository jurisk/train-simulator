mod cameras;
mod debug;
mod world;

use bevy::prelude::App;
use bevy::DefaultPlugins;

use crate::cameras::CameraPlugin;
use crate::debug::DebugPlugin;
use crate::world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WorldPlugin, CameraPlugin, DebugPlugin))
        .run();
}
