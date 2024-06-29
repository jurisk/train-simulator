use std::fmt::{Debug, Formatter};
use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};
use shared_util::coords_xz::CoordsXZ;
use shared_util::direction_xz::DirectionXZ;

use crate::vertex_coords_xz::VertexCoordsXZ;

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub struct TileCoordsXZ {
    pub x: i32,
    pub z: i32,
}

impl TileCoordsXZ {
    pub const ZERO: TileCoordsXZ = TileCoordsXZ::new(0, 0);

    #[must_use]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub const fn from_usizes(x: usize, z: usize) -> Self {
        Self {
            x: x as i32,
            z: z as i32,
        }
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
    pub fn vertex_coords_nw(self) -> VertexCoordsXZ {
        let coords: CoordsXZ = self.into();
        VertexCoordsXZ::from(coords)
    }

    #[must_use]
    pub fn vertex_coords_ne(self) -> VertexCoordsXZ {
        let coords: CoordsXZ = self.into();
        VertexCoordsXZ::from(coords + DirectionXZ::East)
    }

    #[must_use]
    pub fn vertex_coords_se(self) -> VertexCoordsXZ {
        let coords: CoordsXZ = self.into();
        VertexCoordsXZ::from(coords + DirectionXZ::South + DirectionXZ::East)
    }

    #[must_use]
    pub fn vertex_coords_sw(self) -> VertexCoordsXZ {
        let coords: CoordsXZ = self.into();
        VertexCoordsXZ::from(coords + DirectionXZ::South)
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
        (
            self.vertex_coords_nw(),
            self.vertex_coords_ne(),
            self.vertex_coords_se(),
            self.vertex_coords_sw(),
        )
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
        CoordsXZ::new(tile_coords_xz.x, tile_coords_xz.z)
    }
}

impl From<CoordsXZ> for TileCoordsXZ {
    fn from(coords_xz: CoordsXZ) -> Self {
        Self::new(coords_xz.x, coords_xz.z)
    }
}

impl Add<DirectionXZ> for TileCoordsXZ {
    type Output = Self;

    fn add(self, rhs: DirectionXZ) -> Self::Output {
        let direction_coords: CoordsXZ = rhs.into();
        Self {
            x: self.x + direction_coords.x,
            z: self.z + direction_coords.z,
        }
    }
}

impl Add<TileCoordsXZ> for TileCoordsXZ {
    type Output = Self;

    fn add(self, rhs: TileCoordsXZ) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            z: self.z + rhs.z,
        }
    }
}

impl Sub<TileCoordsXZ> for TileCoordsXZ {
    type Output = Self;

    fn sub(self, rhs: TileCoordsXZ) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            z: self.z - rhs.z,
        }
    }
}
