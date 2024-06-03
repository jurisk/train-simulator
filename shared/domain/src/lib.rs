use serde::{Deserialize, Serialize};
use shared_util::coords_xz::CoordsXZ;
use shared_util::random::generate_random_string;
use uuid::Uuid;

pub mod game_state;
pub mod map_level;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ClientId(pub Uuid);

impl ClientId {
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
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
    pub north_west_vertex_xz: CoordsXZ,
    pub building_type:        BuildingType,
}
