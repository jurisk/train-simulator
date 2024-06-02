use serde::{Deserialize, Serialize};

use crate::map_level::MapLevel;

#[derive(Clone, Serialize, Deserialize)]
pub struct GameState {
    pub map_level: MapLevel,
}
