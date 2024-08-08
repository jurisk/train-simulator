use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::server_response::PlayerInfo;
use crate::PlayerId;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PlayerState {
    infos: HashMap<PlayerId, PlayerInfo>,
}

impl PlayerState {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            infos: HashMap::new(),
        }
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

    pub fn insert(&mut self, player_info: PlayerInfo) {
        self.infos.insert(player_info.id, player_info);
    }

    pub fn update_many(&mut self, player_infos: &Vec<PlayerInfo>) {
        for player_info in player_infos {
            self.insert(player_info.clone());
        }
    }

    // Later:   Consider if there even is `remove` - perhaps it is just going disconnected, but we
    //          don't remove the information we had.
    pub fn remove(&mut self, player_id: PlayerId) {
        self.infos.remove(&player_id);
    }

    #[must_use]
    pub fn get(&self, player_id: PlayerId) -> Option<&PlayerInfo> {
        self.infos.get(&player_id)
    }
}
