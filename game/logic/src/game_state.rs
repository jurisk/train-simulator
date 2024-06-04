use std::collections::HashMap;

use shared_domain::map_level::MapLevel;
use shared_domain::{BuildingInfo, PlayerId, PlayerName};

#[derive(Debug, Clone)]
pub struct GameState {
    pub map_level: MapLevel,
    pub buildings: Vec<BuildingInfo>,
    pub players:   HashMap<PlayerId, PlayerName>,
}
