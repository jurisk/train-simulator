use std::collections::HashMap;

use bevy::asset::Assets;
use bevy::math::Vec3;
use bevy::prelude::{Cuboid, Handle, Mesh, Sphere};
use shared_domain::map_level::zoning::ZoningType;

use crate::util::shift_mesh;

pub struct MapAssets {
    fallback:      Handle<Mesh>,
    zoning_meshes: HashMap<ZoningType, Handle<Mesh>>,
}

impl MapAssets {
    #[must_use]
    #[expect(clippy::items_after_statements)]
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        let fallback = meshes.add(Mesh::from(Sphere::default()));
        let mut zoning_meshes = HashMap::new();

        const ZONING_HEIGHT: f32 = 0.001;
        for zoning_type in ZoningType::all() {
            let mut mesh = Mesh::from(Cuboid::new(3.0, ZONING_HEIGHT, 3.0));
            shift_mesh(&mut mesh, Vec3::new(0.0, ZONING_HEIGHT / 2.0, 0.0));
            let mesh = meshes.add(mesh);
            zoning_meshes.insert(zoning_type, mesh);
        }

        Self {
            fallback,
            zoning_meshes,
        }
    }

    #[must_use]
    pub fn zoning_mesh_for(&self, zoning_type: ZoningType) -> Handle<Mesh> {
        match self.zoning_meshes.get(&zoning_type) {
            None => self.fallback.clone(),
            Some(found) => found.clone(),
        }
    }
}
