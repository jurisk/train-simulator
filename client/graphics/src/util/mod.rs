use bevy::prelude::{Mesh, Vec3};
use bevy::render::mesh::VertexAttributeValues;

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
