use bevy::asset::Assets;
use bevy::core::Name;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{default, BuildChildren, Color, Commands, Entity, ResMut, Transform};
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::server_response::Colour;
use shared_domain::transport::transport_location::TransportLocation;
use shared_domain::transport::transport_type::TrainComponentType;
use shared_domain::TransportId;

use crate::game::transport::assets::TransportAssets;
use crate::game::transport::train_layout::calculate_train_component_head_tails_and_final_tail_position;
use crate::game::transport::TransportIndexComponent;

fn transform_from_head_and_tail(head: Vec3, tail: Vec3) -> Transform {
    let direction = (head - tail).normalize(); // Recalculating with new tail

    let midpoint = (head + tail) / 2.0;

    let mut transform = Transform::from_translation(midpoint);
    transform.align(Vec3::Z, direction, Vec3::Y, Vec3::Y);
    transform
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

#[expect(clippy::similar_names, clippy::too_many_arguments)]
pub(crate) fn create_train(
    transport_id: TransportId,
    colour: Colour,
    transport_location: &TransportLocation,
    train_components: &[TrainComponentType],
    commands: &mut Commands,
    transport_assets: &TransportAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
) -> Entity {
    let color = Color::srgb_u8(colour.r, colour.g, colour.b);

    let transforms =
        calculate_train_component_transforms(train_components, transport_location, map_level);

    let mut children = vec![];
    for (idx, train_component_type) in train_components.iter().enumerate() {
        let component = create_train_component(
            idx,
            color,
            *train_component_type,
            commands,
            transport_assets,
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

fn create_train_component(
    index: usize,
    color: Color,
    train_component_type: TrainComponentType,
    commands: &mut Commands,
    transport_assets: &TransportAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    transform: Transform,
) -> Entity {
    let mesh = transport_assets.train_component_mesh_for(train_component_type);

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
