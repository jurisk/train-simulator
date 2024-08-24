use bevy::app::App;
use bevy::asset::Asset;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
    MaterialPlugin, StandardMaterial,
};
use bevy::prelude::{default, AssetServer, Handle, Image, Mesh, Plugin, Reflect, Res};
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};
use shared_domain::map_level::map_level::TerrainType;

use crate::game::map_level::terrain::land::ATTRIBUTE_TERRAIN_TYPE;

pub(crate) struct AdvancedLandMaterialPlugin;

impl Plugin for AdvancedLandMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, LandExtension>,
        >::default());
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[allow(clippy::struct_field_names)]
pub(crate) struct LandExtension {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
    #[uniform(100)]
    sand_terrain_type:  u32,
    #[uniform(100)]
    grass_terrain_type: u32,
    #[uniform(100)]
    rocks_terrain_type: u32,

    #[texture(101, dimension = "2d_array")]
    #[sampler(102)]
    land_textures: Handle<Image>,
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
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ATTRIBUTE_TERRAIN_TYPE.at_shader_location(8),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

pub(crate) fn create_advanced_land_material(
    asset_server: &Res<AssetServer>,
) -> ExtendedMaterial<StandardMaterial, LandExtension> {
    // Later: Refer to https://bevyengine.org/examples/Shaders/array-texture/ for how to use array textures instead
    let land_textures: Handle<Image> = asset_server.load("textures/land.ktx2");
    ExtendedMaterial {
        base:      StandardMaterial {
            perceptual_roughness: 0.8,
            reflectance: 0.0,
            ..default()
        },
        extension: LandExtension {
            sand_terrain_type: TerrainType::Sand.as_u32(),
            grass_terrain_type: TerrainType::Grass.as_u32(),
            rocks_terrain_type: TerrainType::Rocks.as_u32(),
            land_textures,
        },
    }
}
