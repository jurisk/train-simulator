mod util;

use bevy::app::App;
use bevy::asset::Assets;
use bevy::core::Name;
use bevy::pbr::{AlphaMode, PbrBundle, StandardMaterial};
#[allow(deprecated)]
use bevy::prelude::shape::Plane;
use bevy::prelude::{default, Color, Commands, Mesh, Plugin, ResMut, Startup, Transform};

use crate::terrain::util::mesh_from_height_map_data;

pub(crate) struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_water);
        app.add_systems(Startup, create_land);
    }
}

const SIZE_X: usize = 12;
const SIZE_Z: usize = 10;

#[rustfmt::skip]
const DATA: [[f32; SIZE_X]; SIZE_Z] = [
    [-0.5, -0.5, -0.5, -0.5, -0.5, -1.5, -1.5, -1.5, -1.5, -0.5, -0.5, -0.5],
    [-0.5,  0.5,  0.5,  0.5,  0.5, -0.5, -0.5, -0.5, -0.5,  0.5,  0.5, -0.5],
    [-0.5,  0.5,  1.5,  1.5,  1.5,  0.5,  0.5,  0.5, -0.5,  0.5,  0.5, -0.5],
    [-0.5,  0.5,  1.5,  2.5,  2.5,  1.5,  1.5,  1.5,  1.5,  1.5,  0.5, -0.5],
    [-0.5,  0.5,  1.5,  2.5,  3.5,  2.5,  2.5,  3.5,  2.5,  1.5,  0.5, -0.5],
    [-0.5,  0.5,  1.5,  2.5,  3.5,  3.5,  3.5,  3.5,  2.5,  1.5,  0.5, -0.5],
    [-0.5,  0.5,  1.5,  2.5,  2.5,  2.5,  2.5,  2.5,  2.5,  1.5,  0.5, -0.5],
    [-0.5,  0.5,  1.5,  1.5,  1.5,  1.5,  1.5,  1.5,  1.5,  1.5,  0.5, -0.5],
    [-0.5,  0.5,  0.5,  0.5,  0.5,  0.5,  0.5,  0.5,  0.5,  0.5,  0.5, -0.5],
    [-0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5],
];

#[allow(clippy::cast_precision_loss)]
fn create_water(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    #[allow(deprecated)]
    commands.spawn((
        PbrBundle {
            // TODO: The Plane should no longer be square, so we should probably assemble it as a mesh?
            mesh: meshes.add(Plane::from_size(f32::max(SIZE_X as f32, SIZE_Z as f32))),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba_u8(173, 216, 230, 96),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Name::new("Water"),
    ));
}

#[allow(clippy::cast_precision_loss)]
fn create_land(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let data_slice: Vec<&[f32]> = DATA.iter().map(|row| &row[..]).collect();
    let mesh = mesh_from_height_map_data(
        -(SIZE_X as f32) / 2.0,
        (SIZE_X as f32) / 2.0,
        -(SIZE_Z as f32) / 2.0,
        (SIZE_Z as f32) / 2.0,
        &data_slice,
    );
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::DARK_GREEN),
            transform: Transform::default(),
            ..default()
        },
        Name::new("Land"),
    ));
}
