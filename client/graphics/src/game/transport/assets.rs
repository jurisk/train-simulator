use std::collections::HashMap;

use bevy::asset::Assets;
use bevy::math::Vec3;
use bevy::prelude::{Cuboid, Handle, Mesh, Sphere};
use shared_domain::resource_type::ResourceType;
use shared_domain::transport::transport_type::TrainComponentType;

use crate::util::shift_mesh;

pub struct TransportAssets {
    fallback:               Handle<Mesh>,
    train_component_meshes: HashMap<TrainComponentType, Handle<Mesh>>,
}

const GAP_BETWEEN_TRAIN_COMPONENTS: f32 = 0.05;
const TRAIN_WIDTH: f32 = 0.125;
const TRAIN_EXTRA_HEIGHT: f32 = 0.1;

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

impl TransportAssets {
    #[must_use]
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        let fallback = meshes.add(Mesh::from(Sphere::default()));

        let mut map = HashMap::new();
        map.insert(
            TrainComponentType::Engine,
            // Later: Add also a cylinder
            adjusted_cuboid(
                GAP_BETWEEN_TRAIN_COMPONENTS,
                TRAIN_WIDTH,
                TRAIN_WIDTH * 1.6, // Train engine is higher
                TrainComponentType::Engine.length_in_tiles(),
                TRAIN_EXTRA_HEIGHT,
            ),
        );
        for resource_type in ResourceType::all() {
            let train_component_type = TrainComponentType::Car(resource_type);
            map.insert(
                train_component_type,
                adjusted_cuboid(
                    GAP_BETWEEN_TRAIN_COMPONENTS,
                    TRAIN_WIDTH,
                    TRAIN_WIDTH * 0.4, // Train cars are lower
                    train_component_type.length_in_tiles(),
                    TRAIN_EXTRA_HEIGHT,
                ),
            );
        }

        let train_component_meshes = map.into_iter().map(|(k, v)| (k, meshes.add(v))).collect();

        Self {
            fallback,
            train_component_meshes,
        }
    }

    #[must_use]
    pub fn train_component_mesh_for(
        &self,
        train_component_type: TrainComponentType,
    ) -> Handle<Mesh> {
        match self.train_component_meshes.get(&train_component_type) {
            None => self.fallback.clone(),
            Some(found) => found.clone(),
        }
    }
}
