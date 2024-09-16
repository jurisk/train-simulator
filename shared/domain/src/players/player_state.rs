use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::server_response::{Colour, PlayerInfo};
use crate::{PlayerId, PlayerName};

// TODO: The players are actually 'Nation'-s or 'Polity'-s, and the players just control them.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerState {
    infos: HashMap<PlayerId, PlayerInfo>,
}

impl PlayerState {
    #[must_use]
    pub fn two_players() -> Self {
        let mut result = Self {
            infos: HashMap::new(),
        };
        result.insert(PlayerInfo {
            id:     PlayerId::random(),
            name:   PlayerName("Union".to_string()),
            colour: Colour::rgb(153, 51, 255),
        });
        result.insert(PlayerInfo {
            id:     PlayerId::random(),
            name:   PlayerName("Alliance".to_string()),
            colour: Colour::rgb(255, 51, 51),
        });
        result
    }

    #[must_use]
    pub fn infos(&self) -> Vec<&PlayerInfo> {
        self.infos.values().collect()
    }

    #[must_use]
    pub fn infos_cloned(&self) -> Vec<PlayerInfo> {
        self.infos.values().cloned().collect()
    }

    #[must_use]
    pub fn ids(&self) -> Vec<PlayerId> {
        self.infos.keys().copied().collect()
    }

    fn insert(&mut self, player_info: PlayerInfo) {
        self.infos.insert(player_info.id, player_info);
    }

    #[must_use]
    pub fn get(&self, player_id: PlayerId) -> Option<&PlayerInfo> {
        self.infos.get(&player_id)
    }
}
