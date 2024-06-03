use bimap::BiMap;
use shared_domain::{ClientId, PlayerId};

pub struct ConnectionRegistry {
    map: BiMap<PlayerId, ClientId>,
}

impl ConnectionRegistry {
    pub fn new() -> Self {
        Self { map: BiMap::new() }
    }

    pub fn register(&mut self, player_id: PlayerId, client_id: ClientId) {
        self.map.insert(player_id, client_id);
    }

    pub fn unregister_by_player_id(&mut self, player_id: PlayerId) {
        self.map.remove_by_left(&player_id);
    }

    pub fn unregister_by_client_id(&mut self, client_id: ClientId) {
        self.map.remove_by_right(&client_id);
    }

    #[must_use]
    pub fn get_client_id(&self, player_id: &PlayerId) -> Option<&ClientId> {
        self.map.get_by_left(player_id)
    }

    #[must_use]
    pub fn get_player_id(&self, client_id: &ClientId) -> Option<&PlayerId> {
        self.map.get_by_right(client_id)
    }

    #[must_use]
    pub fn get_all_player_ids(&self) -> Vec<&PlayerId> {
        self.map.left_values().collect()
    }

    #[must_use]
    pub fn get_all_client_ids(&self) -> Vec<&ClientId> {
        self.map.right_values().collect()
    }
}
