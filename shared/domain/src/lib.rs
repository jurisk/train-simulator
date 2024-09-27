#![allow(clippy::unused_self)]

use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use shared_util::random::generate_random_string;
use transport::track_type::TrackType;
use uuid::Uuid;

use crate::tile_coords_xz::TileCoordsXZ;

pub mod building;
pub mod cargo_amount;
pub mod cargo_map;
pub mod client_command;
pub mod directional_edge;
pub mod edge_xz;
pub mod game_state;
pub mod game_time;
pub mod map_level;
pub mod metrics;
pub mod players;
pub mod resource_type;
pub mod scenario;
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
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

impl Display for PlayerName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl UserName {
    #[must_use]
    pub fn random(seed: u64) -> Self {
        Self(generate_random_string(6, seed))
    }

    #[must_use]
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct UserName(String);

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

newtype_uuid!(UserId, "U");

impl UserId {
    #[must_use]
    pub fn hash_to_u64(self) -> u64 {
        let (a, b) = self.0.as_u64_pair();
        a ^ b
    }
}

// TODO HIGH: `PlayerId` could actually have `String` underlying or even be `enum`?
newtype_uuid!(PlayerId, "P");

newtype_uuid!(GameId, "G");
newtype_uuid!(StationId, "S");
newtype_uuid!(IndustryBuildingId, "IB");
newtype_uuid!(TransportId, "T");
newtype_uuid!(ZoningId, "Z");

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord, Debug)]
pub struct ScenarioId(pub String);
impl Default for ScenarioId {
    #[expect(clippy::unwrap_used)]
    fn default() -> Self {
        ScenarioId::all().first().unwrap().clone()
    }
}

impl FromStr for ScenarioId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ScenarioId::all()
            .into_iter()
            .find(|ScenarioId(scenario_id)| scenario_id == s)
            .ok_or(())
    }
}

impl ScenarioId {
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![Self("usa_east".to_string()), Self("europe".to_string())]
    }
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord, Debug)]
pub struct MapId(pub String);

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct TrackId {
    pub tile:       TileCoordsXZ,
    pub track_type: TrackType,
}

impl TrackId {
    #[must_use]
    pub fn new(tile: TileCoordsXZ, track_type: TrackType) -> Self {
        Self { tile, track_type }
    }
}

impl Debug for TrackId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "T-{:?}-{:?}", self.tile, self.track_type)
    }
}
