#![allow(clippy::module_name_repetitions)]

use bevy::app::{App, Plugin};
use bevy::prelude::{Assets, Mesh, ResMut, Resource, Startup};

use crate::game::buildings::assets::BuildingAssets;
use crate::game::buildings::tracks::TrackAssets;
use crate::game::map_level::assets::MapAssets;
use crate::game::transport::assets::TransportAssets;
// Later: Use https://github.com/NiklasEi/bevy_asset_loader? Or perhaps not.

#[derive(Resource)]
pub struct GameAssets {
    pub track_assets:     TrackAssets,
    pub building_assets:  BuildingAssets,
    pub transport_assets: TransportAssets,
    pub map_assets:       MapAssets,
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_assets);
    }
}

fn setup_assets(mut commands: bevy::prelude::Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let meshes = meshes.as_mut();
    commands.insert_resource(GameAssets {
        track_assets:     TrackAssets::new(meshes),
        building_assets:  BuildingAssets::new(meshes),
        transport_assets: TransportAssets::new(meshes),
        map_assets:       MapAssets::new(meshes),
    });
}
