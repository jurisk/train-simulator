use std::fmt::{Debug, Formatter};
use std::ops::Add;

use serde::{Deserialize, Serialize};
use shared_util::coords_xz::CoordsXZ;
use shared_util::direction_xz::DirectionXZ;

use crate::vertex_coords_xz::VertexCoordsXZ;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub struct TileCoordsXZ(pub CoordsXZ);

impl TileCoordsXZ {
    pub const ZERO: TileCoordsXZ = TileCoordsXZ(CoordsXZ::ZERO);

    #[must_use]
    pub fn new(x: i32, z: i32) -> Self {
        Self(CoordsXZ::new(x, z))
    }

    #[must_use]
    pub fn from_usizes(x: usize, z: usize) -> Self {
        Self(CoordsXZ::from_usizes(x, z))
    }

    #[must_use]
    pub fn vertex_coords_clockwise(
        self,
        direction_xz: DirectionXZ,
    ) -> (VertexCoordsXZ, VertexCoordsXZ) {
        let (nw, ne, se, sw) = self.vertex_coords_nw_ne_se_sw();
        match direction_xz {
            DirectionXZ::North => (nw, ne),
            DirectionXZ::East => (ne, se),
            DirectionXZ::South => (se, sw),
            DirectionXZ::West => (sw, nw),
        }
    }

    #[must_use]
    pub fn vertex_coords(self) -> [VertexCoordsXZ; 4] {
        let (nw, ne, se, sw) = self.vertex_coords_nw_ne_se_sw();
        [nw, ne, se, sw]
    }

    #[must_use]
    pub fn vertex_coords_nw_ne_se_sw(
        self,
    ) -> (
        VertexCoordsXZ,
        VertexCoordsXZ,
        VertexCoordsXZ,
        VertexCoordsXZ,
    ) {
        let coords: CoordsXZ = self.into();
        let nw = VertexCoordsXZ::from(coords);
        let ne = VertexCoordsXZ::from(coords + DirectionXZ::East);
        let se = VertexCoordsXZ::from(coords + DirectionXZ::South + DirectionXZ::East);
        let sw = VertexCoordsXZ::from(coords + DirectionXZ::South);
        (nw, ne, se, sw)
    }
}

impl Debug for TileCoordsXZ {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let coords = <TileCoordsXZ as Into<CoordsXZ>>::into(*self);
        write!(f, "T-{}-{}", coords.x, coords.z)
    }
}

impl From<TileCoordsXZ> for CoordsXZ {
    fn from(tile_coords_xz: TileCoordsXZ) -> Self {
        tile_coords_xz.0
    }
}

impl From<CoordsXZ> for TileCoordsXZ {
    fn from(coords_xz: CoordsXZ) -> Self {
        Self(coords_xz)
    }
}

impl Add<DirectionXZ> for TileCoordsXZ {
    type Output = TileCoordsXZ;

    fn add(self, rhs: DirectionXZ) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Add<TileCoordsXZ> for TileCoordsXZ {
    type Output = TileCoordsXZ;

    fn add(self, rhs: TileCoordsXZ) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
