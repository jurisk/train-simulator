use bevy::app::App;
use bevy::asset::Asset;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
    MaterialPlugin, StandardMaterial,
};
use bevy::prelude::{default, AssetServer, Handle, Image, Mesh, Plugin, Reflect, Res};
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};

use crate::level::terrain::land::domain::TerrainType::{Grass, Rocks, Sand, SeaBottom};
use crate::level::terrain::land::ATTRIBUTE_TERRAIN_TYPE;

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
    sea_bottom_terrain_type: u32,
    #[uniform(100)]
    sand_terrain_type:       u32,
    #[uniform(100)]
    grass_terrain_type:      u32,
    #[uniform(100)]
    rocks_terrain_type:      u32,

    #[texture(101)]
    #[sampler(102)]
    sea_bottom_texture: Handle<Image>,

    #[texture(103)]
    #[sampler(104)]
    sand_texture: Handle<Image>,

    #[texture(105)]
    #[sampler(106)]
    grass_texture: Handle<Image>,

    #[texture(107)]
    #[sampler(108)]
    rocks_texture: Handle<Image>,
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

pub(crate) fn create_advanced_land_material(
    asset_server: &Res<AssetServer>,
) -> ExtendedMaterial<StandardMaterial, LandExtension> {
    let sea_bottom_texture: Handle<Image> = asset_server.load("textures/sand.jpg");
    let sand_texture: Handle<Image> = asset_server.load("textures/sand.jpg");
    let grass_texture: Handle<Image> = asset_server.load("textures/grass.jpg");
    let rocks_texture: Handle<Image> = asset_server.load("textures/rock.jpg");
    ExtendedMaterial {
        base:      StandardMaterial { ..default() },
        extension: LandExtension {
            sea_bottom_terrain_type: SeaBottom as u32,
            sand_terrain_type: Sand as u32,
            grass_terrain_type: Grass as u32,
            rocks_terrain_type: Rocks as u32,
            sea_bottom_texture,
            sand_texture,
            grass_texture,
            rocks_texture,
        },
    }
}
