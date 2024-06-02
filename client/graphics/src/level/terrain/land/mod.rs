use bevy::app::App;
use bevy::asset::{AssetServer, Assets};
use bevy::core::Name;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::{
    default, Color, Commands, MaterialMeshBundle, Mesh, OnEnter, Plugin, Res, ResMut,
    StandardMaterial, Transform,
};
use bevy::render::mesh::MeshVertexAttribute;
use bevy::render::render_resource::VertexFormat;

use crate::level::terrain::land::advanced_land_material::{
    create_advanced_land_material, AdvancedLandMaterialPlugin, LandExtension,
};
use crate::level::terrain::land::domain::TerrainType;
use crate::level::terrain::land::tiled_mesh_from_height_map_data::tiled_mesh_from_height_map_data;
use crate::level::terrain::Y_COEF;
use crate::level::GameStateResource;
use crate::states::ClientState;

mod advanced_land_material;
mod domain;
mod stretched_mesh_from_height_map_data;
mod tiled_mesh_from_height_map_data;

pub(crate) struct LandPlugin;

impl Plugin for LandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AdvancedLandMaterialPlugin);
        app.add_systems(OnEnter(ClientState::Playing), create_land);
        // Eventually, clean-up will be also needed
    }
}

const ATTRIBUTE_TERRAIN_TYPE: MeshVertexAttribute =
    MeshVertexAttribute::new("TerrainType", 988_540_917, VertexFormat::Uint32);

#[allow(unused)]
enum LandMaterialType {
    Advanced,
    Debug,
}

const LAND_MATERIAL_TYPE: LandMaterialType = LandMaterialType::Advanced;

#[allow(
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value,
    clippy::cast_lossless
)]
pub(crate) fn create_land(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut advanced_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, LandExtension>>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    game_state_resource: Res<GameStateResource>,
) {
    let level = &game_state_resource.game_state.level;
    let data_slice: Vec<Vec<f32>> = level
        .terrain
        .height_map
        .iter()
        .map(|row| row.iter().map(|h| h.0 as f32).collect::<Vec<_>>())
        .collect();

    let half_x = (level.terrain.size_x as f32) / 2.0;
    let half_z = (level.terrain.size_z as f32) / 2.0;
    let height_map = &level.terrain.height_map;

    let mut mesh = tiled_mesh_from_height_map_data(
        -half_x,
        half_x,
        -half_z,
        half_z,
        Y_COEF,
        data_slice,
        ATTRIBUTE_TERRAIN_TYPE,
        |x, z| TerrainType::from_height(height_map[z][x]) as u32,
    );

    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    let mesh = meshes.add(mesh);

    let transform = Transform::default();

    match LAND_MATERIAL_TYPE {
        LandMaterialType::Advanced => {
            let material = advanced_materials.add(create_advanced_land_material(&asset_server));
            commands.spawn((
                MaterialMeshBundle {
                    mesh,
                    material,
                    transform,
                    ..default()
                },
                Name::new("Land"),
            ));
        },
        LandMaterialType::Debug => {
            let material = standard_materials.add(StandardMaterial {
                perceptual_roughness: 0.8,
                reflectance: 0.0,
                base_color: Color::GRAY,
                ..default()
            });
            commands.spawn((
                MaterialMeshBundle {
                    mesh,
                    material,
                    transform,
                    ..default()
                },
                Name::new("Land"),
            ));
        },
    };
}
