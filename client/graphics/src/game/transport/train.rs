use bevy::asset::Assets;
use bevy::core::Name;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{
    default, BuildChildren, Color, Commands, Cuboid, Entity, Mesh, ResMut, Transform,
};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::PlayerInfo;
use shared_domain::transport_info::TransportLocation;
use shared_domain::transport_type::TrainComponentType;
use shared_domain::TransportId;
use shared_util::geometry::rotation_aligned_with_direction;

use crate::game::transport::train_layout::calculate_train_component_head_tails_and_final_tail_position;
use crate::game::transport::TransportIndexComponent;
use crate::util::shift_mesh;

const GAP_BETWEEN_TRAIN_COMPONENTS: f32 = 0.05;
const TRAIN_WIDTH: f32 = 0.125;
const TRAIN_EXTRA_HEIGHT: f32 = 0.1;

fn transform_from_head_and_tail(head: Vec3, tail: Vec3) -> Transform {
    let direction = (head - tail).normalize(); // Recalculating with new tail

    let midpoint = (head + tail) / 2.0;
    Transform {
        rotation: rotation_aligned_with_direction(direction),
        translation: midpoint,
        ..default()
    }
}

pub(crate) fn calculate_train_component_transforms(
    train_components: &[TrainComponentType],
    transport_location: &TransportLocation,
    map_level: &MapLevel,
) -> Vec<Transform> {
    let (head_tails, _) = calculate_train_component_head_tails_and_final_tail_position(
        train_components,
        transport_location,
        map_level,
    );

    head_tails
        .into_iter()
        .map(|(head, tail)| transform_from_head_and_tail(head, tail))
        .collect()
}

#[allow(clippy::similar_names, clippy::too_many_arguments)]
pub(crate) fn create_train(
    transport_id: TransportId,
    player_info: &PlayerInfo,
    transport_location: &TransportLocation,
    train_components: &[TrainComponentType],
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
) -> Entity {
    let colour = player_info.colour;
    let color = Color::rgb_u8(colour.r, colour.g, colour.b);

    let transforms =
        calculate_train_component_transforms(train_components, transport_location, map_level);

    let mut children = vec![];
    for (idx, train_component_type) in train_components.iter().enumerate() {
        let component = create_train_component(
            idx,
            color,
            *train_component_type,
            commands,
            meshes,
            materials,
            transforms[idx],
        );
        children.push(component);
    }

    let parent = commands
        .spawn(Name::new(format!("Train {transport_id:?}")))
        .id();

    commands.entity(parent).push_children(&children);
    parent
}

fn adjusted_cuboid(
    z_gap: f32,
    x_length: f32,
    y_length: f32,
    z_length: f32,
    height_from_ground: f32,
) -> Mesh {
    let mut mesh = Mesh::from(Cuboid::new(x_length, y_length, z_length - z_gap * 2.0));

    shift_mesh(
        &mut mesh,
        Vec3::new(0.0, height_from_ground + y_length / 2.0, 0.0),
    );

    mesh
}

#[allow(clippy::too_many_arguments, clippy::items_after_statements)]
fn create_train_component(
    index: usize,
    color: Color,
    train_component_type: TrainComponentType,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    transform: Transform,
) -> Entity {
    let mesh = match train_component_type {
        TrainComponentType::Engine => {
            // TODO: Add also a cylinder
            adjusted_cuboid(
                GAP_BETWEEN_TRAIN_COMPONENTS,
                TRAIN_WIDTH,
                TRAIN_WIDTH * 1.6, // Train engine is higher
                train_component_type.length_in_tiles(),
                TRAIN_EXTRA_HEIGHT,
            )
        },
        TrainComponentType::Car => {
            adjusted_cuboid(
                GAP_BETWEEN_TRAIN_COMPONENTS,
                TRAIN_WIDTH,
                TRAIN_WIDTH * 0.4, // Train cars are lower
                train_component_type.length_in_tiles(),
                TRAIN_EXTRA_HEIGHT,
            )
        },
    };

    let mesh = meshes.add(mesh);

    let entity_commands = commands.spawn((
        PbrBundle {
            material: materials.add(color),
            transform,
            mesh,
            ..default()
        },
        TransportIndexComponent(index),
        Name::new(format!("{train_component_type:?}-{index}")),
    ));

    entity_commands.id()
}
