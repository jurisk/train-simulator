use bevy::app::App;
use bevy::prelude::Plugin;

use crate::level::domain::Level;
use crate::level::terrain::TerrainPlugin;

mod domain;
pub mod terrain;

pub(crate) struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        let terrain_heights = vec![
            vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1],
            vec![1, 2, 2, 2, 2, 1, 1, 1, 1, 2, 2, 1],
            vec![1, 2, 3, 3, 3, 2, 2, 2, 1, 2, 2, 1],
            vec![1, 2, 3, 4, 4, 3, 3, 3, 3, 3, 2, 1],
            vec![1, 2, 3, 4, 5, 4, 4, 5, 4, 3, 2, 1],
            vec![1, 2, 3, 4, 5, 5, 5, 5, 4, 3, 2, 1],
            vec![1, 2, 3, 4, 4, 4, 4, 4, 4, 3, 2, 1],
            vec![1, 2, 3, 3, 3, 3, 3, 3, 3, 3, 2, 1],
            vec![1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1],
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];

        let level = Level::new(terrain_heights, 1, 2);

        // TODO: Load from a web service or resources instead
        app.insert_resource(level);
        app.add_plugins(TerrainPlugin);
    }
}
