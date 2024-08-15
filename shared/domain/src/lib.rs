#![allow(clippy::unused_self)]

use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use shared_util::random::generate_random_string;
use transport::track_type::TrackType;
use uuid::Uuid;

use crate::tile_coords_xz::TileCoordsXZ;

pub mod building;
mod cargo_amount;
pub mod cargo_map;
pub mod client_command;
pub mod edge_xz;
pub mod game_state;
pub mod game_time;
pub mod map_level;
pub mod players;
pub mod resource_type;
pub mod server_response;
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

macro_rules! newtype_uuid {
    ($name:ident, $prefix:expr) => {
        #[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
        pub struct $name(Uuid);

        impl $name {
            #[must_use]
            pub fn new(uuid: Uuid) -> Self {
                Self(uuid)
            }

            #[must_use]
            pub fn random() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let uuid_str = self.0.to_string();
                let truncated_uuid = &uuid_str[.. 8];
                write!(f, "{}-{}", $prefix, truncated_uuid)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::from_str(s).map(Self)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

newtype_uuid!(PlayerId, "P");

impl PlayerId {
    #[must_use]
    pub fn hash_to_u64(self) -> u64 {
        let (a, b) = self.0.as_u64_pair();
        a ^ b
    }
}

newtype_uuid!(GameId, "G");
newtype_uuid!(TrackId, "T");
newtype_uuid!(StationId, "S");
newtype_uuid!(IndustryBuildingId, "IB");
newtype_uuid!(TransportId, "T");
newtype_uuid!(ZoningId, "Z");

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord, Debug)]
pub struct MapId(pub String);

impl Default for MapId {
    #[allow(clippy::unwrap_used)]
    fn default() -> Self {
        MapId::all().first().unwrap().clone()
    }
}

impl MapId {
    #[must_use]
    pub fn all() -> Vec<MapId> {
        vec![
            MapId("sample".to_string()),
            MapId("europe".to_string()),
            MapId("usa".to_string()),
        ]
    }
}

impl FromStr for MapId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        MapId::all()
            .into_iter()
            .find(|MapId(map_id)| map_id == s)
            .ok_or(())
    }
}
