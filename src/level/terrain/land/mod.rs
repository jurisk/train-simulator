use bevy::app::App;
use bevy::asset::Assets;
use bevy::core::Name;
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::{
    default, Color, Commands, MaterialMeshBundle, Mesh, OnEnter, Plugin, Res, ResMut,
    StandardMaterial, Transform,
};
use bevy::render::mesh::MeshVertexAttribute;
use bevy::render::render_resource::VertexFormat;
use util::mesh_from_height_map_data;

use crate::level::terrain::land::advanced_land_material::{
    create_advanced_land_material, AdvancedLandMaterialPlugin, LandExtension,
};
use crate::level::terrain::land::domain::TerrainType;
use crate::level::terrain::Y_COEF;
use crate::level::LevelResource;
use crate::states::GameState;

mod advanced_land_material;
mod domain;
mod util;

pub(crate) struct LandPlugin;

impl Plugin for LandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AdvancedLandMaterialPlugin);
        app.add_systems(OnEnter(GameState::Playing), create_land);
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

const LAND_MATERIAL_TYPE: LandMaterialType = LandMaterialType::Debug;

#[allow(
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value,
    clippy::cast_lossless
)]
pub(crate) fn create_land(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut advanced_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, LandExtension>>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    level_resource: Res<LevelResource>,
) {
    let level = &level_resource.level;
    let data_slice: Vec<Vec<f32>> = level
        .terrain
        .height_map
        .iter()
        .map(|row| row.iter().map(|h| h.0 as f32).collect::<Vec<_>>())
        .collect();

    let half_x = (level.terrain.size_x as f32) / 2.0;
    let half_z = (level.terrain.size_z as f32) / 2.0;
    let height_map = &level.terrain.height_map;

    // This allows to change the terrain type depending on something else than the height (e.g. part of the level definition)
    let terrain_types: Vec<_> = height_map
        .iter()
        .flat_map(|row| {
            row.iter()
                .map(|h| TerrainType::from_height(*h) as u32)
                .collect::<Vec<_>>()
        })
        .collect();

    let mut mesh = mesh_from_height_map_data(-half_x, half_x, -half_z, half_z, Y_COEF, data_slice)
        .with_inserted_attribute(ATTRIBUTE_TERRAIN_TYPE, terrain_types);

    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    let mesh = meshes.add(mesh);
    let transform = Transform::default();

    match LAND_MATERIAL_TYPE {
        LandMaterialType::Advanced => {
            let material = advanced_materials.add(create_advanced_land_material());
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
                base_color: Color::rgba(0.1, 0.9, 0.1, 1.0),
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
