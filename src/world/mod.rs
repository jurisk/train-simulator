mod cameras;
mod lights;
mod terrain;

use crate::world::cameras::CameraPlugin;
use crate::world::lights::LightsPlugin;
use crate::world::terrain::TerrainPlugin;
use bevy::app::App;
use bevy::prelude::Plugin;

pub(crate) struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((LightsPlugin, CameraPlugin, TerrainPlugin));
    }
}
