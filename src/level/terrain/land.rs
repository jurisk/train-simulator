use bevy::app::App;
use bevy::asset::Assets;
use bevy::core::Name;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::{
    default, Asset, Commands, MaterialMeshBundle, MaterialPlugin, Mesh, OnEnter, Plugin, Reflect,
    Res, ResMut, StandardMaterial, Transform,
};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

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
pub(crate) struct LandExtension {
    // We need to ensure that the bindings of the base material and the extension do not conflict,
    // so we start from binding slot 100, leaving slots 0-99 for the base material.
    #[uniform(100)]
    max_y: f32,
}

impl MaterialExtension for LandExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/land.wgsl".into()
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
    let mesh = mesh_from_height_map_data(-half_x, half_x, -half_z, half_z, Y_COEF, data_slice);

    let material = ExtendedMaterial {
        base:      StandardMaterial { ..default() },
        extension: LandExtension {
            // TODO:    Detect it. And this is not "max_y", but some threshold for when mountains start.
            //          And we need more thresholds, for water-sand, sand-grass, grass-rocks.
            max_y: 4.0,
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
