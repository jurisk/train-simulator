#![allow(clippy::module_name_repetitions)]

use bevy::app::{App, Plugin};
use bevy::prelude::{Assets, Mesh, ResMut, Resource, Startup};

use crate::game::buildings::tracks::TrackAssets;

// Later: Use https://github.com/NiklasEi/bevy_asset_loader? Or perhaps not.

// TODO HIGH: Use such pre-loaded assets instead of creating a new mesh / material every time.
#[derive(Resource)]
pub struct GameAssets {
    pub track_assets: TrackAssets,
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_assets);
    }
}

fn setup_assets(mut commands: bevy::prelude::Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(GameAssets {
        track_assets: TrackAssets::new(meshes.as_mut()),
    });
}
