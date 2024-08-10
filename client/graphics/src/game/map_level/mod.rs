use bevy::prelude::{App, Plugin};

use crate::game::map_level::terrain::TerrainPlugin;
use crate::game::map_level::zoning::ZoningPlugin;

pub mod assets;
pub mod terrain;
mod zoning;

pub(crate) struct MapLevelPlugin;

impl Plugin for MapLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerrainPlugin);
        app.add_plugins(ZoningPlugin);
    }
}
