use std::collections::HashMap;

use crate::map_level::MapLevel;
use crate::{BuildingInfo, PlayerId, PlayerName};

// TODO: Move to `game/logic` - this should not be a DTO!
#[derive(Debug, Clone)]
pub struct GameState {
    pub map_level: MapLevel,
    pub buildings: Vec<BuildingInfo>,
    pub players:   HashMap<PlayerId, PlayerName>,
}
