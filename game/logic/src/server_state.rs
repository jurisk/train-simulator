use std::collections::HashMap;

use shared_domain::game_state::GameState;
use shared_domain::GameId;

use crate::connection_registry::ConnectionRegistry;

#[derive(Default)]
pub struct ServerState {
    pub connection_registry: ConnectionRegistry,
    pub games:               HashMap<GameId, GameState>,
}

impl ServerState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            connection_registry: ConnectionRegistry::new(),
            games:               HashMap::new(),
        }
    }
}
