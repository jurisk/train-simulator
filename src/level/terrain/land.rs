use bevy::app::App;
use bevy::asset::Assets;
use bevy::core::Name;
use bevy::prelude::{
    default, Asset, Commands, Material, MaterialMeshBundle, MaterialPlugin, Mesh, OnEnter, Plugin,
    Res, ResMut, Transform, TypePath,
};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

use crate::level::terrain::util::mesh_from_height_map_data;
use crate::level::terrain::Y_COEF;
use crate::level::LevelResource;
use crate::states::GameState;

pub(crate) struct LandPlugin;

impl Plugin for LandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<LandMaterial>::default());
        app.add_systems(OnEnter(GameState::Playing), create_land);
        // Eventually, clean-up will be also needed
    }
}

// TODO: Apply lighting and shadows by reusing PBR, e.g., `pbr_input_from_standard_material`
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct LandMaterial {}

impl Material for LandMaterial {
    // fn vertex_shader() -> ShaderRef {
    //     "shaders/land_shader.wgsl".into()
    // }

    fn fragment_shader() -> ShaderRef {
        "shaders/land_shader.wgsl".into()
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
    mut materials: ResMut<Assets<LandMaterial>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
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
    let mesh = mesh_from_height_map_data(-half_x, half_x, -half_z, half_z, Y_COEF, data_slice);

    let material = LandMaterial {};
    // let material = StandardMaterial {
    //     base_color: Color::DARK_GREEN,
    //     ..default()
    // };

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
