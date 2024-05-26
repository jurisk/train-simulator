use bevy::prelude::{App, Plugin};

use crate::level::terrain::land::LandPlugin;
use crate::level::terrain::water::WaterPlugin;

pub mod land;
mod util;
pub mod water;

pub(crate) struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LandPlugin);
        app.add_plugins(WaterPlugin);
    }
}

const Y_COEF: f32 = 0.2;
