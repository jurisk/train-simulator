use bevy::prelude::{App, Plugin};

use crate::game::map_level::terrain::land::LandPlugin;
use crate::game::map_level::terrain::water::WaterPlugin;

pub mod land;
pub mod water;

pub(crate) struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LandPlugin);
        app.add_plugins(WaterPlugin);
    }
}
