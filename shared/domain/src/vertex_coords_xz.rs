use std::fmt::{Debug, Formatter};
use std::ops::Add;

use serde::{Deserialize, Serialize};
use shared_util::coords_xz::CoordsXZ;
use shared_util::direction_xz::DirectionXZ;

use crate::TileCoordsXZ;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub struct VertexCoordsXZ(pub CoordsXZ);

impl Debug for VertexCoordsXZ {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "V{:?}", <VertexCoordsXZ as Into<CoordsXZ>>::into(*self))
    }
}

impl VertexCoordsXZ {
    #[must_use]
    pub fn from_usizes(x: usize, z: usize) -> Self {
        Self(CoordsXZ::from_usizes(x, z))
    }

    #[must_use]
    pub fn to_tile_coords_xz(self) -> TileCoordsXZ {
        let coords: CoordsXZ = self.into();
        coords.into()
    }

    #[must_use]
    pub fn from_tile_coords_xz(tile_coords_xz: TileCoordsXZ) -> Self {
        let coords: CoordsXZ = tile_coords_xz.into();
        coords.into()
    }
}

impl From<VertexCoordsXZ> for CoordsXZ {
    fn from(vertex_coords_xz: VertexCoordsXZ) -> Self {
        vertex_coords_xz.0
    }
}

impl From<CoordsXZ> for VertexCoordsXZ {
    fn from(coords_xz: CoordsXZ) -> Self {
        Self(coords_xz)
    }
}

impl Add<DirectionXZ> for VertexCoordsXZ {
    type Output = Self;

    fn add(self, rhs: DirectionXZ) -> Self::Output {
        Self(self.0 + rhs)
    }
}
