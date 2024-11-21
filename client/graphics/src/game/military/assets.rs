use std::collections::HashMap;

use bevy::prelude::{Assets, Color, Mesh, StandardMaterial};
use shared_domain::military::ShellType;

use crate::assets::MeshAndMaterial;
use crate::util::shapes::generate_cone;

pub struct MilitaryAssets {
    pub(crate) shells: ShellAssets,
}

impl MilitaryAssets {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            shells: ShellAssets::new(meshes, materials),
        }
    }
}

pub struct ShellAssets {
    fallback:             MeshAndMaterial,
    shell_meshes_by_type: HashMap<ShellType, MeshAndMaterial>,
}

impl ShellAssets {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let mesh = generate_cone(1.0, 2.0, 16);
        let mesh = meshes.add(mesh);

        let color = Color::srgb(1.0, 0.0, 0.0);
        let material = materials.add(color);

        let mesh_and_material = MeshAndMaterial { mesh, material };

        let shell_meshes_by_type =
            HashMap::from([(ShellType::Standard, mesh_and_material.clone())]);

        Self {
            fallback: mesh_and_material,
            shell_meshes_by_type,
        }
    }

    pub(crate) fn mesh_and_material_for_shell_type(
        &self,
        shell_type: ShellType,
    ) -> MeshAndMaterial {
        self.shell_meshes_by_type
            .get(&shell_type)
            .unwrap_or(&self.fallback)
            .clone()
    }
}
