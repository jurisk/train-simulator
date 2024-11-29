pub mod shapes;

use bevy::prelude::{Mesh, Transform, Vec3};
use bevy::render::mesh::VertexAttributeValues;

#[must_use]
pub fn transform_from_midpoint_and_direction(midpoint: Vec3, direction: Vec3) -> Transform {
    let mut transform = Transform::from_translation(midpoint);
    transform.align(Vec3::Z, direction.normalize(), Vec3::Y, Vec3::Y);
    transform
}

pub fn shift_mesh(mesh: &mut Mesh, shift: Vec3) {
    if let Some(VertexAttributeValues::Float32x3(ref mut positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for position in positions.iter_mut() {
            position[0] += shift.x;
            position[1] += shift.y;
            position[2] += shift.z;
        }
    }
}
