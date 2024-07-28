#![allow(clippy::unused_self)]

use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use shared_util::random::generate_random_string;
use transport::track_type::TrackType;
use uuid::Uuid;

use crate::building_type::BuildingType;
use crate::tile_coords_xz::TileCoordsXZ;

pub mod building_info;
pub mod building_state;
pub mod building_type;
mod cargo_amount;
pub mod cargo_map;
pub mod client_command;
pub mod edge_xz;
pub mod game_state;
pub mod game_time;
pub mod map_level;
pub mod production_type;
pub mod resource_type;
pub mod server_response;
pub mod station_type;
pub mod terrain;
pub mod tile_coords_xz;
pub mod tile_coverage;
pub mod transport;
pub mod vertex_coords_xz;
pub mod water;

// Later: We initially wanted it to be Uuid, but bevy_simplenet uses u128, so we can stick with that for now for easier compatibility
#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ClientId(u128);

impl Debug for ClientId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let uuid_str = self.0.to_string();
        let truncated_uuid = &uuid_str[.. 8];
        write!(f, "C-{truncated_uuid}")
    }
}

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
pub struct PlayerName(String);

impl PlayerName {
    #[must_use]
    pub fn random(seed: u64) -> Self {
        Self(generate_random_string(6, seed))
    }

    #[must_use]
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

impl Display for PlayerName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct PlayerId(Uuid);

impl Debug for PlayerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let uuid_str = self.0.to_string();
        let truncated_uuid = &uuid_str[.. 8];
        write!(f, "P-{truncated_uuid}")
    }
}

impl PlayerId {
    #[must_use]
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }

    #[must_use]
    pub fn hash_to_u64(self) -> u64 {
        let (a, b) = self.0.as_u64_pair();
        a ^ b
    }
}

impl FromStr for PlayerId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(Self)
    }
}

impl Display for PlayerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct GameId(Uuid);

impl Debug for GameId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let uuid_str = self.0.to_string();
        let truncated_uuid = &uuid_str[.. 8];
        write!(f, "G-{truncated_uuid}")
    }
}

impl GameId {
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Copy, Clone, Hash)]
pub struct BuildingId(Uuid);

impl BuildingId {
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Debug for BuildingId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let uuid_str = self.0.to_string();
        let truncated_uuid = &uuid_str[.. 8];
        write!(f, "B-{truncated_uuid}")
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub struct TransportId(Uuid);

impl Debug for TransportId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let uuid_str = self.0.to_string();
        let truncated_uuid = &uuid_str[.. 8];
        write!(f, "T-{truncated_uuid}")
    }
}

impl TransportId {
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}
