#![allow(clippy::unused_self)]

use std::cmp::Ordering;
use std::collections::HashSet;
use std::f32::consts::SQRT_2;
use std::fmt::{Debug, Formatter};
use std::ops::Add;

use bevy_math::Vec3;
use serde::{Deserialize, Serialize};
use shared_util::coords_xz::CoordsXZ;
use shared_util::direction_xz::DirectionXZ;
use shared_util::random::generate_random_string;
use uuid::Uuid;

use crate::building_state::BuildingState;
use crate::terrain::Terrain;

pub mod building_state;
pub mod client_command;
pub mod map_level;
pub mod server_response;
pub mod terrain;

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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct TransportId(pub Uuid);

impl TransportId {
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

// Later: Possibly rename to `ConnectionType` or something. And `TrackType` thus has multiple of these `ConnectionType`-s.
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
    #[allow(clippy::match_same_arms, clippy::missing_panics_doc)]
    #[must_use]
    pub fn other_end(self, direction: DirectionXZ) -> DirectionXZ {
        match (self, direction) {
            (TrackType::NorthEast, DirectionXZ::North) => DirectionXZ::East,
            (TrackType::NorthEast, DirectionXZ::East) => DirectionXZ::North,
            (TrackType::NorthSouth, DirectionXZ::North) => DirectionXZ::South,
            (TrackType::NorthSouth, DirectionXZ::South) => DirectionXZ::North,
            (TrackType::NorthWest, DirectionXZ::North) => DirectionXZ::West,
            (TrackType::NorthWest, DirectionXZ::West) => DirectionXZ::North,
            (TrackType::EastWest, DirectionXZ::East) => DirectionXZ::West,
            (TrackType::EastWest, DirectionXZ::West) => DirectionXZ::East,
            (TrackType::SouthEast, DirectionXZ::South) => DirectionXZ::East,
            (TrackType::SouthEast, DirectionXZ::East) => DirectionXZ::South,
            (TrackType::SouthWest, DirectionXZ::South) => DirectionXZ::West,
            (TrackType::SouthWest, DirectionXZ::West) => DirectionXZ::South,
            _ => {
                panic!("Invalid track type {self:?} and direction {direction:?} combination",)
            },
        }
    }

    #[must_use]
    pub fn relative_tiles_used(self) -> HashSet<TileCoordsXZ> {
        HashSet::from([TileCoordsXZ::ZERO])
    }

    #[must_use]
    pub fn connections(self) -> HashSet<DirectionXZ> {
        let (a, b) = self.connections_clockwise();
        HashSet::from([a, b])
    }

    #[must_use]
    pub fn connections_clockwise(self) -> (DirectionXZ, DirectionXZ) {
        match self {
            TrackType::NorthEast => (DirectionXZ::North, DirectionXZ::East),
            TrackType::NorthSouth => (DirectionXZ::North, DirectionXZ::South),
            TrackType::NorthWest => (DirectionXZ::West, DirectionXZ::North),
            TrackType::EastWest => (DirectionXZ::East, DirectionXZ::West),
            TrackType::SouthEast => (DirectionXZ::East, DirectionXZ::South),
            TrackType::SouthWest => (DirectionXZ::South, DirectionXZ::West),
        }
    }

    #[must_use]
    pub fn length_in_tiles(self) -> f32 {
        match self {
            TrackType::NorthSouth | TrackType::EastWest => 1.0,
            TrackType::NorthEast
            | TrackType::NorthWest
            | TrackType::SouthEast
            | TrackType::SouthWest => SQRT_2 / 2.0,
        }
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

    #[must_use]
    pub fn contains(&self, tile: TileCoordsXZ) -> bool {
        match self {
            TileCoverage::Single(single_tile) => *single_tile == tile,
            TileCoverage::Multiple(tiles) => tiles.contains(&tile),
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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum TrainComponentType {
    Engine,
    Car,
}

impl TrainComponentType {
    #[must_use]
    pub fn length_in_tiles(self) -> f32 {
        match self {
            TrainComponentType::Engine => 0.8,
            TrainComponentType::Car => 0.4,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum TransportType {
    Train(Vec<TrainComponentType>),
    RoadVehicle,
    Ship,
}

impl TransportType {
    #[must_use]
    pub fn length_in_tiles(&self) -> f32 {
        match self {
            TransportType::Train(components) => {
                components
                    .iter()
                    .map(|component| component.length_in_tiles())
                    .sum()
            },
            TransportType::RoadVehicle => todo!(),
            TransportType::Ship => todo!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct TileTrack {
    pub tile_coords_xz: TileCoordsXZ,
    pub track_type:     TrackType,
}

impl TileTrack {
    #[must_use]
    pub fn progress_coordinates(
        &self,
        pointing_in: DirectionXZ,
        progress_within_tile: ProgressWithinTile,
        terrain: &Terrain,
    ) -> Vec3 {
        let (entry, exit) = terrain.entry_and_exit(pointing_in, self);
        let track_length = (exit - entry).length();
        let direction = (exit - entry).normalize();
        entry + direction * progress_within_tile.progress() * track_length
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct ProgressWithinTile(f32);

impl Eq for ProgressWithinTile {}

impl PartialOrd for ProgressWithinTile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProgressWithinTile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or_else(|| panic!("Failed to compare {self:?} and {other:?}"))
    }
}

impl ProgressWithinTile {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(progress: f32) -> Self {
        assert!(
            (0.0 ..= 1.0).contains(&progress),
            "Progress must be between 0.0 and 1.0, but was {progress}"
        );
        Self(progress)
    }

    #[must_use]
    pub fn from_point_between_two_points(start_end: (Vec3, Vec3), point: Vec3) -> Self {
        let (start, end) = start_end;
        let value = (point - start).length() / (end - start).length();
        Self::new(value)
    }

    #[must_use]
    pub fn just_entering() -> Self {
        Self(0.0)
    }

    #[must_use]
    pub fn about_to_exit() -> Self {
        Self(1.0)
    }

    #[must_use]
    pub fn out_of_bounds(self) -> bool {
        self.progress() >= 1.0
    }

    #[must_use]
    pub fn progress(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportLocation {
    pub pointing_in:          DirectionXZ,
    pub tile_path:            Vec<TileTrack>, /* Which tile is it on now, and which tiles has it been on - only as much as to cover the vehicle's length */
    pub progress_within_tile: ProgressWithinTile,
}

impl TransportLocation {
    #[must_use]
    pub fn entering_from(&self) -> DirectionXZ {
        let track_type = self.tile_path[0].track_type;
        track_type.other_end(self.pointing_in)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportVelocity {
    pub tiles_per_second: f32,
}

// TODO: Later - this needs to be developed into proper path-finding
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum MovementOrders {
    Stop,
    TemporaryPickFirst,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportInfo {
    pub transport_id:    TransportId,
    pub owner_id:        PlayerId,
    pub location:        TransportLocation,
    pub transport_type:  TransportType,
    pub velocity:        TransportVelocity,
    pub movement_orders: MovementOrders,
}

impl TransportInfo {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::items_after_statements
    )]
    fn jump_tile(&mut self, building_state: &BuildingState) {
        let current_tile_track = self.location.tile_path[0];
        let next_tile_coords = current_tile_track.tile_coords_xz + self.location.pointing_in;
        let tracks_at_next_tile: Vec<TrackType> = building_state.track_types_at(next_tile_coords);
        let reversed = self.location.pointing_in.reverse();
        let valid_tracks_at_next_tile: Vec<TrackType> = tracks_at_next_tile
            .into_iter()
            .filter(|track_type| track_type.connections().contains(&reversed))
            .collect();
        // TODO: Support other strategies
        assert_eq!(self.movement_orders, MovementOrders::TemporaryPickFirst);
        let next_track_type = valid_tracks_at_next_tile[0];
        let next_tile_track = TileTrack {
            tile_coords_xz: next_tile_coords,
            track_type:     next_track_type,
        };

        self.location.tile_path.insert(0, next_tile_track);

        // Later: We are rather crudely sometimes removing the last element when we are inserting an
        // element.
        // This means - depending on `HEURISTIC_COEF` - that sometimes we will be carrying around
        // "too many tiles", or it could lead to running out of tiles if it is too short.
        // The alternative is to use `calculate_train_component_head_tails_and_final_tail_position`
        // to calculate the tail position, and then remove the last tiles if they are not needed,
        // but that introduces more complexity.
        const HEURISTIC_COEF: f32 = 2.0;
        if self.location.tile_path.len()
            > (HEURISTIC_COEF * self.transport_type.length_in_tiles()) as usize
        {
            let _ = self.location.tile_path.pop();
        }

        self.location.progress_within_tile.0 -= 1.0;
        self.location.pointing_in = next_track_type.other_end(reversed);
    }

    fn normalise_progress_jumping_tiles(&mut self, building_state: &BuildingState) {
        while self.location.progress_within_tile.out_of_bounds() {
            self.jump_tile(building_state);
        }
    }

    // TODO: Also invoke on the server-side, authoritative!
    pub fn advance(&mut self, seconds: f32, building_state: &BuildingState) {
        let track_type = self.location.tile_path[0].track_type;
        let track_length = track_type.length_in_tiles();
        let TransportVelocity { tiles_per_second } = self.velocity;
        let ProgressWithinTile(progress_within_tile) = self.location.progress_within_tile;
        let effective_speed = tiles_per_second / track_length;
        let new_progress = progress_within_tile + effective_speed * seconds;
        let new_progress = ProgressWithinTile(new_progress);
        self.location.progress_within_tile = new_progress;
        self.normalise_progress_jumping_tiles(building_state);
    }
}
