use std::collections::HashMap;

use bevy::prelude::{Assets, Cuboid, Handle, Mesh, Sphere};
use shared_domain::production_type::ProductionType;
use shared_domain::station_type::{StationOrientation, StationType};

const PRODUCTION_HEIGHT: f32 = 0.5;
const STATION_HEIGHT: f32 = 0.01;

pub struct BuildingAssets {
    fallback:          Handle<Mesh>,
    production_meshes: HashMap<ProductionType, Handle<Mesh>>,
    station_meshes:    HashMap<StationType, Handle<Mesh>>,
}

impl BuildingAssets {
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        // TODO: Use `shift_mesh` to actually avoid creating double-height meshes, just shift them
        let fallback = meshes.add(Mesh::from(Sphere::default()));

        let mut production_meshes = HashMap::new();

        for production_type in ProductionType::all() {
            let mesh = meshes.add(Mesh::from(Cuboid::new(3.0, PRODUCTION_HEIGHT * 2.0, 3.0)));
            production_meshes.insert(production_type, mesh);
        }

        let mut station_meshes = HashMap::new();

        for station_type in StationType::all() {
            let (x, z) = match station_type.orientation {
                StationOrientation::NorthToSouth => {
                    (
                        station_type.platforms as f32,
                        station_type.length_in_tiles as f32,
                    )
                },
                StationOrientation::EastToWest => {
                    (
                        station_type.length_in_tiles as f32,
                        station_type.platforms as f32,
                    )
                },
            };
            let mesh = meshes.add(Mesh::from(Cuboid::new(x, STATION_HEIGHT * 2.0, z)));
            station_meshes.insert(station_type, mesh);
        }

        Self {
            fallback,
            production_meshes,
            station_meshes,
        }
    }

    #[must_use]
    pub fn production_mesh_for(&self, production_type: ProductionType) -> Handle<Mesh> {
        match self.production_meshes.get(&production_type) {
            None => self.fallback.clone(),
            Some(found) => found.clone(),
        }
    }

    #[must_use]
    pub fn station_mesh_for(&self, station_type: StationType) -> Handle<Mesh> {
        match self.station_meshes.get(&station_type) {
            None => self.fallback.clone(),
            Some(found) => found.clone(),
        }
    }
}
