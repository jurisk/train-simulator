use serde::{Deserialize, Serialize};

use crate::level::Level;

#[derive(Clone, Serialize, Deserialize)]
pub struct GameState {
    pub level: Level,
}
