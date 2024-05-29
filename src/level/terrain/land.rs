use bevy::app::App;
use bevy::asset::Assets;
use bevy::core::Name;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
};
use bevy::prelude::{
    default, Asset, Commands, MaterialMeshBundle, MaterialPlugin, Mesh, OnEnter, Plugin, Reflect,
    Res, ResMut, StandardMaterial, Transform,
};
use bevy::render::mesh::{MeshVertexAttribute, MeshVertexBufferLayout};
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError, VertexFormat,
};

use crate::level::domain::Height;
use crate::level::terrain::land::TerrainType::{Grass, Rocks, Sand, SeaBottom};
use crate::level::terrain::util::mesh_from_height_map_data;
use crate::level::terrain::Y_COEF;
use crate::level::LevelResource;
use crate::states::GameState;

pub(crate) struct LandPlugin;

impl Plugin for LandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, LandExtension>,
        >::default());
        app.add_systems(OnEnter(GameState::Playing), create_land);
        // Eventually, clean-up will be also needed
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[allow(clippy::struct_field_names)]
pub(crate) struct LandExtension {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
    #[uniform(100)]
    sea_bottom_terrain_type: u32,
    #[uniform(100)]
    sand_terrain_type:       u32,
    #[uniform(100)]
    grass_terrain_type:      u32,
    #[uniform(100)]
    rocks_terrain_type:      u32,
}

impl MaterialExtension for LandExtension {
    fn vertex_shader() -> ShaderRef {
        "shaders/land.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/land.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ATTRIBUTE_TERRAIN_TYPE.at_shader_location(8),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

const ATTRIBUTE_TERRAIN_TYPE: MeshVertexAttribute =
    MeshVertexAttribute::new("TerrainType", 988_540_917, VertexFormat::Uint32);

#[repr(u32)]
enum TerrainType {
    SeaBottom = 0,
    Sand      = 1,
    Grass     = 2,
    Rocks     = 3,
}

fn terrain_type(height: Height) -> TerrainType {
    if height.0 <= 7 {
        SeaBottom
    } else if height.0 <= 9 {
        Sand
    } else if height.0 < 12 {
        Grass
    } else {
        Rocks
    }
}

#[allow(
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value,
    clippy::cast_lossless
)]
pub(crate) fn create_land(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, LandExtension>>>,
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
                .map(|h| terrain_type(*h) as u32)
                .collect::<Vec<_>>()
        })
        .collect();

    let mut mesh = mesh_from_height_map_data(-half_x, half_x, -half_z, half_z, Y_COEF, data_slice)
        .with_inserted_attribute(ATTRIBUTE_TERRAIN_TYPE, terrain_types);

    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    let material = ExtendedMaterial {
        base:      StandardMaterial { ..default() },
        extension: LandExtension {
            sea_bottom_terrain_type: SeaBottom as u32,
            sand_terrain_type:       Sand as u32,
            grass_terrain_type:      Grass as u32,
            rocks_terrain_type:      Rocks as u32,
        },
    };

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(mesh),
            material: materials.add(material),
            transform: Transform::default(),
            ..default()
        },
        Name::new("Land"),
    ));
}
