use bevy::asset::{AssetServer, Assets};
use bevy::color::palettes::css::GRAY;
use bevy::core::Name;
use bevy::pbr::{ExtendedMaterial, MeshMaterial3d};
use bevy::picking::mesh_picking::RayCastPickable;
use bevy::prelude::{
    App, Color, Commands, EventReader, FixedUpdate, MaterialMeshBundle, Mesh, Mesh3d, Plugin, Res,
    ResMut, StandardMaterial, Transform, default,
};
use bevy::render::mesh::MeshVertexAttribute;
use bevy::render::render_resource::VertexFormat;
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, ServerResponse};
use shared_domain::vertex_coords_xz::VertexCoordsXZ;
use shared_util::grid_xz::GridXZ;

use crate::communication::domain::ServerMessageEvent;
use crate::game::map_level::terrain::land::advanced_land_material::{
    AdvancedLandMaterialPlugin, LandExtension, create_advanced_land_material,
};
use crate::game::map_level::terrain::land::tiled_mesh_from_height_map_data::tiled_mesh_from_height_map_data;

mod advanced_land_material;
pub mod tiled_mesh_from_height_map_data;

pub(crate) struct LandPlugin;

impl Plugin for LandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AdvancedLandMaterialPlugin);
        app.add_systems(FixedUpdate, handle_map_level_updated);
    }
}

pub(crate) const ATTRIBUTE_TERRAIN_TYPE: MeshVertexAttribute =
    MeshVertexAttribute::new("TerrainType", 988_540_917, VertexFormat::Uint32);

#[expect(unused)]
enum LandMaterialType {
    Advanced,
    Debug,
}

const LAND_MATERIAL_TYPE: LandMaterialType = LandMaterialType::Advanced;

#[expect(clippy::needless_pass_by_value, clippy::collapsible_match)]
fn handle_map_level_updated(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut advanced_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, LandExtension>>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            if let GameResponse::GameJoined(_player_id, game_state) = game_response {
                let map_level = game_state.map_level();
                create_land(
                    &mut commands,
                    &mut meshes,
                    &asset_server,
                    &mut advanced_materials,
                    &mut standard_materials,
                    map_level,
                );
            }
        }
    }
}

#[expect(clippy::cast_precision_loss)]
pub(crate) fn create_land(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    asset_server: &Res<AssetServer>,
    advanced_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, LandExtension>>>,
    standard_materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
) {
    let terrain = map_level.terrain();
    let data_slice: GridXZ<VertexCoordsXZ, f32> = GridXZ::filled_with(
        terrain.vertex_count_x(),
        terrain.vertex_count_z(),
        f32::default(),
    )
    .map_with_coords(|coords, _| terrain.height_at(coords.into()).as_f32() * terrain.y_coef());

    let half_x = (terrain.tile_count_x() as f32) / 2.0;
    let half_z = (terrain.tile_count_z() as f32) / 2.0;

    let (tiles, mesh) = tiled_mesh_from_height_map_data(
        -half_x,
        half_x,
        -half_z,
        half_z,
        data_slice,
        ATTRIBUTE_TERRAIN_TYPE,
        |coords: VertexCoordsXZ| terrain.terrain_at(coords) as u32,
    );

    commands.insert_resource(tiles);

    // Previously, we did mesh.duplicate_vertices() here, but I didn't figure out if it helps or
    // hurts performance. At one point it was mandatory as we also did duplicate.calculate_face_normals().

    let mesh = meshes.add(mesh);

    let transform = Transform::default();

    match LAND_MATERIAL_TYPE {
        LandMaterialType::Advanced => {
            let material = advanced_materials.add(create_advanced_land_material(asset_server));
            commands.spawn((
                MaterialMeshBundle {
                    mesh: Mesh3d(mesh),
                    material: MeshMaterial3d(material),
                    transform,
                    ..default()
                },
                RayCastPickable,
                Name::new("Land"),
            ));
        },
        LandMaterialType::Debug => {
            let material = standard_materials.add(StandardMaterial {
                perceptual_roughness: 0.8,
                reflectance: 0.0,
                base_color: Color::Srgba(GRAY),
                ..default()
            });
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                transform,
                RayCastPickable,
                Name::new("Land"),
            ));
        },
    };
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec3;
    use shared_domain::map_level::map_level::Height;
    use shared_domain::map_level::terrain::{DEFAULT_Y_COEF, Terrain};

    use super::*;

    #[test]
    fn test_logical_to_world() {
        let vertex_heights = vec![vec![0, 1, 2], vec![3, 1, 5], vec![6, 7, 2]];
        let vertex_heights = GridXZ::new(vertex_heights).map(|&height| Height::from_u8(height));

        let terrain = Terrain::new(DEFAULT_Y_COEF, vertex_heights, GridXZ::new(vec![]));

        assert_eq!(
            terrain.logical_to_world(VertexCoordsXZ::from_usizes(0, 0)),
            Vec3::new(-1.0, 0.0, -1.0)
        );
        assert_eq!(
            terrain.logical_to_world(VertexCoordsXZ::from_usizes(1, 1)),
            Vec3::new(0.0, terrain.y_coef(), 0.0)
        );
        assert_eq!(
            terrain.logical_to_world(VertexCoordsXZ::from_usizes(2, 2)),
            Vec3::new(1.0, 2.0 * terrain.y_coef(), 1.0)
        );
    }
}
