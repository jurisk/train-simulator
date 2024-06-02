use bevy::prelude::Plugin;
use crate::game::buildings::BuildingsPlugin;
use crate::game::map_level::MapLevelPlugin;

mod buildings;
mod map_level;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(BuildingsPlugin);
        app.add_plugins(MapLevelPlugin);
    }
}