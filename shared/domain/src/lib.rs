#![allow(clippy::unused_self)]

use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::ops::Add;

use serde::{Deserialize, Serialize};
use shared_util::coords_xz::CoordsXZ;
use shared_util::direction_xz::DirectionXZ;
use shared_util::random::generate_random_string;
use uuid::Uuid;

pub mod client_command;
pub mod map_level;
pub mod server_response;

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

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub struct TileCoordsXZ(pub CoordsXZ);

impl TileCoordsXZ {
    pub const ZERO: TileCoordsXZ = TileCoordsXZ(CoordsXZ::ZERO);

    #[must_use]
    pub fn from_usizes(x: usize, z: usize) -> Self {
        Self(CoordsXZ::from_usizes(x, z))
    }

    #[must_use]
    pub fn north_west_vertex(self) -> VertexCoordsXZ {
        let coords: CoordsXZ = self.into();
        coords.into()
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
    NorthEast,
    NorthSouth,
    NorthWest,
    EastWest,
    SouthEast,
    SouthWest,
}

impl TrackType {
    #[must_use]
    fn relative_tiles_used(self) -> HashSet<TileCoordsXZ> {
        HashSet::from([TileCoordsXZ::ZERO])
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum ProductionType {
    CoalMine,
    IronMine,
    IronWorks,
}

impl ProductionType {
    #[must_use]
    fn relative_tiles_used(self) -> HashSet<TileCoordsXZ> {
        HashSet::new() // TODO: Implement
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum BuildingType {
    Track(TrackType),
    Production(ProductionType),
}

impl BuildingType {
    #[must_use]
    fn relative_tiles_used(self) -> HashSet<TileCoordsXZ> {
        match self {
            BuildingType::Track(track_type) => track_type.relative_tiles_used(),
            BuildingType::Production(production_type) => production_type.relative_tiles_used(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct BuildingInfo {
    pub owner_id:             PlayerId,
    pub building_id:          BuildingId,
    pub north_west_vertex_xz: VertexCoordsXZ,
    pub building_type:        BuildingType,
}

impl BuildingInfo {
    #[must_use]
    fn base_tile(&self) -> TileCoordsXZ {
        self.north_west_vertex_xz.to_tile_coords_xz()
    }

    #[must_use]
    pub fn tiles_used(&self) -> HashSet<TileCoordsXZ> {
        self.building_type
            .relative_tiles_used()
            .into_iter()
            .map(|diff| self.base_tile() + diff)
            .collect()
    }
}
