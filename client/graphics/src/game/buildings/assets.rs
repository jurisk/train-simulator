use std::collections::HashMap;

use bevy::prelude::{Assets, Cuboid, Handle, Mesh, Sphere, Vec3};
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::station_type::{StationOrientation, StationType};

use crate::util::shift_mesh;

const PRODUCTION_HEIGHT: f32 = 0.5;
const STATION_HEIGHT: f32 = 0.01;

pub struct BuildingAssets {
    fallback:        Handle<Mesh>,
    industry_meshes: HashMap<IndustryType, Handle<Mesh>>,
    station_meshes:  HashMap<StationType, Handle<Mesh>>,
}

impl BuildingAssets {
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        let fallback = meshes.add(Mesh::from(Sphere::default()));

        let mut industry_meshes = HashMap::new();

        for industry_type in IndustryType::all() {
            let mut mesh = Mesh::from(Cuboid::new(3.0, PRODUCTION_HEIGHT, 3.0));
            shift_mesh(&mut mesh, Vec3::new(0.0, PRODUCTION_HEIGHT / 2.0, 0.0));
            let mesh = meshes.add(mesh);
            industry_meshes.insert(industry_type, mesh);
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
            industry_meshes,
            station_meshes,
        }
    }

    #[must_use]
    pub fn industry_mesh_for(&self, industry_type: IndustryType) -> Handle<Mesh> {
        match self.industry_meshes.get(&industry_type) {
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
