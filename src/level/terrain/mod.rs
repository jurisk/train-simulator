use bevy::app::App;
use bevy::prelude::{OnEnter, Plugin};

use crate::level::terrain::land::create_land;
use crate::level::terrain::water::create_water;
use crate::states::GameState;

mod land;
mod util;
mod water;

pub(crate) struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), create_water);
        app.add_systems(OnEnter(GameState::Playing), create_land);
        // Eventually, clean-up will be also needed
    }
}

const Y_COEF: f32 = 0.2;
