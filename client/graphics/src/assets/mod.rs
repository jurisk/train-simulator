#![allow(clippy::module_name_repetitions)]

use bevy::app::{App, Plugin};
use bevy::prelude::{Assets, Handle, Mesh, ResMut, Resource, StandardMaterial, Startup};

use crate::game::buildings::assets::BuildingAssets;
use crate::game::buildings::tracks::assets::TrackAssets;
use crate::game::map_level::assets::MapAssets;
use crate::game::military::assets::MilitaryAssets;
use crate::game::transport::assets::TransportAssets;
// Later: Use https://github.com/NiklasEi/bevy_asset_loader? Or perhaps not.

#[derive(Resource)]
pub struct GameAssets {
    pub track_assets:     TrackAssets,
    pub building_assets:  BuildingAssets,
    pub transport_assets: TransportAssets,
    pub map_assets:       MapAssets,
    pub military_assets:  MilitaryAssets,
}

#[derive(Clone)]
pub struct MeshAndMaterial {
    pub mesh:     Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_assets);
    }
}

fn setup_assets(
    mut commands: bevy::prelude::Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let meshes = meshes.as_mut();
    let materials = materials.as_mut();
    commands.insert_resource(GameAssets {
        track_assets:     TrackAssets::new(meshes),
        building_assets:  BuildingAssets::new(meshes),
        transport_assets: TransportAssets::new(meshes),
        map_assets:       MapAssets::new(meshes),
        military_assets:  MilitaryAssets::new(meshes, materials),
    });
}
