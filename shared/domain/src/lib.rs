use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use shared_util::coords_xz::CoordsXZ;
use shared_util::random::generate_random_string;
use uuid::Uuid;

pub mod client_command;
pub mod map_level;
pub mod server_response;

#[derive(Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
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
}

impl From<VertexCoordsXZ> for CoordsXZ {
    fn from(vertex_coords_xz: VertexCoordsXZ) -> Self {
        vertex_coords_xz.0
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub struct TileCoordsXZ(pub CoordsXZ);

impl TileCoordsXZ {
    #[must_use]
    pub fn from_usizes(x: usize, z: usize) -> Self {
        Self(CoordsXZ::from_usizes(x, z))
    }
}

impl Debug for TileCoordsXZ {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "T{:?}", <TileCoordsXZ as Into<CoordsXZ>>::into(*self))
    }
}

impl From<TileCoordsXZ> for CoordsXZ {
    fn from(tile_coords_xz: TileCoordsXZ) -> Self {
        tile_coords_xz.0
    }
}

// Later: We initially wanted it to be Uuid, but renet uses u64, so we can stick with that for now for easier compatibility
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ClientId(u64);

impl ClientId {
    #[must_use]
    pub fn random() -> Self {
        Self(fastrand::u64(.. u64::MAX))
    }

    #[must_use]
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[must_use]
    pub fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct PlayerName(pub String);

impl PlayerName {
    #[must_use]
    pub fn random() -> Self {
        Self(generate_random_string(5))
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct PlayerId(pub Uuid);

impl PlayerId {
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct GameId(pub Uuid);

impl GameId {
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct BuildingId(pub Uuid);

impl BuildingId {
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum TrackType {
    NorthSouth,
    EastWest,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum ProductionType {
    CoalMine,
    IronMine,
    IronWorks,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum BuildingType {
    Track(TrackType),
    Production(ProductionType),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct BuildingInfo {
    pub building_id:          BuildingId,
    // TODO: OK, but which direction is North-West according to our coordinate system? Let us define it somewhere as a Direction class?
    pub north_west_vertex_xz: VertexCoordsXZ,
    pub building_type:        BuildingType,
}
