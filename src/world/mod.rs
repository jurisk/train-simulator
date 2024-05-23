mod lights;
mod terrain;

use bevy::app::App;
use bevy::prelude::Plugin;

use crate::world::lights::LightsPlugin;
use crate::world::terrain::TerrainPlugin;

pub(crate) struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((LightsPlugin, TerrainPlugin));
    }
}
