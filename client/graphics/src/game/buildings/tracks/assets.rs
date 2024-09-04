use std::collections::HashMap;

use bevy::asset::{Assets, Handle};
use bevy::log::{info, warn};
use bevy::math::Vec3;
use bevy::prelude::{Cuboid, Mesh};
use bigdecimal::BigDecimal;
use shared_domain::map_level::terrain::DEFAULT_Y_COEF;

use crate::game::buildings::tracks::positions::pick_rail_positions;

const RAIL_DIAMETER: f32 = 0.025;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RailLengthKey(BigDecimal);

impl RailLengthKey {
    #[must_use]
    fn for_vectors(a: Vec3, b: Vec3) -> Self {
        let length_squared = (b - a).length_squared();

        let length_squared = BigDecimal::try_from(length_squared).unwrap_or_else(|e| {
            warn!("Could not convert length squared to BigDecimal: {e}");
            BigDecimal::from(1)
        });

        // Note.    Not sure if we have bugs in how we pre-populate the rail meshes, but this
        //          rounding was important for the meshes to be found.
        let rounded = length_squared.round(1);

        Self(rounded)
    }
}

pub struct TrackAssets {
    fallback:           Handle<Mesh>,
    rail_meshes_by_key: HashMap<RailLengthKey, Handle<Mesh>>,
}

impl TrackAssets {
    #[must_use]
    pub(crate) fn new(meshes: &mut Assets<Mesh>) -> Self {
        let mut rail_meshes_by_key = HashMap::new();

        // For the diagonal rails
        let (a1, a2) = pick_rail_positions(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        let (b1, b2) = pick_rail_positions(Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 1.0));

        // This is suboptimal, as it is tied to `DEFAULT_Y_COEF` instead of dynamically taking it from `Terrain`.
        let nominals = vec![
            (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
            (
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, DEFAULT_Y_COEF, 0.0),
            ),
            (a1, b2),
            (a2, b1),
        ];

        for (a, b) in nominals {
            let key = RailLengthKey::for_vectors(a, b);
            let length = (b - a).length();
            let length_squared = (b - a).length_squared();

            // Later: Make the rails round, they will look nicer. Look at Rise of Industry, for example.
            let handle = meshes.add(Mesh::from(Cuboid::new(
                RAIL_DIAMETER,
                RAIL_DIAMETER,
                length,
            )));

            info!(
                "Registering rail mesh for key {key:?} ({a:?}, {b:?}, l = {length}, l_sq = {length_squared})"
            );

            rail_meshes_by_key.insert(key, handle);
        }

        let fallback = meshes.add(Mesh::from(Cuboid::default()));

        Self {
            fallback,
            rail_meshes_by_key,
        }
    }

    #[must_use]
    pub(crate) fn rail_mesh_for(&self, a: Vec3, b: Vec3) -> Handle<Mesh> {
        let key = RailLengthKey::for_vectors(a, b);
        match self.rail_meshes_by_key.get(&key) {
            None => {
                let length = (b - a).length();
                let length_squared = (b - a).length_squared();
                let known_keys: Vec<_> = self.rail_meshes_by_key.keys().collect();
                warn!(
                    "Rail mesh not found for length {length}: key {key:?} ({a:?}, {b:?}, l_sq = {length_squared}), using fallback. Known keys: {known_keys:?}"
                );
                self.fallback.clone()
            },
            Some(found) => found.clone(),
        }
    }
}
