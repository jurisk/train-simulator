use bevy::app::App;
use bevy::prelude::Plugin;

use crate::level::domain::Level;
use crate::level::terrain::TerrainPlugin;

mod domain;
pub mod terrain;

pub(crate) struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        let level_json = include_str!("../../assets/levels/default.json");
        let level = serde_json::from_str::<Level>(level_json)
            .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));

        app.insert_resource(level);
        app.add_plugins(TerrainPlugin);
    }
}
