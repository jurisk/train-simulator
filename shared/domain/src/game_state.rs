use serde::{Deserialize, Serialize};

use crate::map_level::MapLevel;
use crate::BuildingInfo;

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct GameState {
    pub map_level: MapLevel,
    pub buildings: Vec<BuildingInfo>,
}
