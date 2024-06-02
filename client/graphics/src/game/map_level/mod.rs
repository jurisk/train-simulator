use bevy::app::App;
use bevy::prelude::Plugin;

use crate::game::map_level::terrain::TerrainPlugin;

pub mod terrain;

pub(crate) struct MapLevelPlugin;

impl Plugin for MapLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerrainPlugin);
    }
}
