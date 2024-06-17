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

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub struct TileCoordsXZ(pub CoordsXZ);

impl TileCoordsXZ {
    pub const ZERO: TileCoordsXZ = TileCoordsXZ(CoordsXZ::ZERO);

    #[must_use]
    pub fn from_usizes(x: usize, z: usize) -> Self {
        Self(CoordsXZ::from_usizes(x, z))
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

// Later: We initially wanted it to be Uuid, but bevy_simplenet uses u128, so we can stick with that for now for easier compatibility
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ClientId(u128);

#[allow(clippy::cast_possible_truncation)]
impl ClientId {
    #[must_use]
    pub fn random() -> Self {
        Self::from_u128(fastrand::u128(.. u128::MAX))
    }

    #[must_use]
    pub fn from_u128(raw: u128) -> Self {
        Self(raw)
    }

    #[must_use]
    pub fn as_u128(self) -> u128 {
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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct BuildingId(pub Uuid);

impl BuildingId {
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct VehicleId(pub Uuid);

impl VehicleId {
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

// TODO: Possibly rename to `ConnectionType` or something. And `TrackType` thus has multiple of these `ConnectionType`-s.
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
pub enum StationOrientation {
    NorthSouth,
    EastWest,
}

// TODO: Build some test stations in test setup
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct StationType {
    orientation:     StationOrientation,
    platforms:       usize,
    length_in_tiles: usize,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum BuildingType {
    Track(TrackType),
    Station(StationType),
    Production(ProductionType),
}

impl BuildingType {
    #[must_use]
    #[allow(unused)] // TODO: Start using eventually
    fn relative_tiles_used(self) -> HashSet<TileCoordsXZ> {
        match self {
            BuildingType::Track(track_type) => track_type.relative_tiles_used(),
            BuildingType::Production(production_type) => production_type.relative_tiles_used(),
            BuildingType::Station(_station_type) => todo!(), // TODO: Implement
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum TileCoverage {
    Single(TileCoordsXZ),
    Multiple(HashSet<TileCoordsXZ>),
}

impl TileCoverage {
    #[must_use]
    pub fn to_set(&self) -> HashSet<TileCoordsXZ> {
        match self {
            TileCoverage::Single(tile) => HashSet::from([*tile]),
            TileCoverage::Multiple(tiles) => tiles.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct BuildingInfo {
    pub owner_id:      PlayerId,
    pub building_id:   BuildingId,
    pub covers_tiles:  TileCoverage,
    pub building_type: BuildingType,
}

// TODO: Reconsider if perhaps `Train` is `Transport` and consists of `TrainEngine`-s and `TrainCar`-s? Improve test setup to immediately test this.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum VehicleType {
    TrainEngine,
    TrainCar,
}

impl VehicleType {
    #[must_use]
    pub fn length_in_tiles(self) -> f32 {
        match self {
            VehicleType::TrainEngine => 0.8,
            VehicleType::TrainCar => 0.6,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct VehicleInfo {
    pub vehicle_id:   VehicleId,
    pub owner_id:     PlayerId,
    // TODO: Rethink `location`, as vehicles can travel between tiles, and also a `TrainCar` can directly follow a `TrainEngine` or another `TrainCar`
    pub location:     TileCoordsXZ, /* TODO: Probably have a sub-location float with `0` meaning "just about to enter the tile" and `1` meaning "just about to leave the tile". */
    // TODO: I think for trains we may need a whole list of `TileCoordsXZ, TrackType` pairs, as a train can travel through multiple tiles and multiple track types.
    pub direction:    DirectionXZ,
    pub vehicle_type: VehicleType,
    // TODO: Velocity?
}
