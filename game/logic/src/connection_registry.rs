#![allow(dead_code, clippy::trivially_copy_pass_by_ref)]

use bimap::BiMap;
use shared_domain::{ClientId, UserId};

#[derive(Default)]
pub(crate) struct ConnectionRegistry {
    map: BiMap<UserId, ClientId>,
}

impl ConnectionRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self { map: BiMap::new() }
    }

    pub fn register(&mut self, user_id: UserId, client_id: ClientId) {
        self.map.insert(user_id, client_id);
    }

    pub fn unregister_by_user_id(&mut self, user_id: &UserId) {
        self.map.remove_by_left(user_id);
    }

    pub fn unregister_by_client_id(&mut self, client_id: &ClientId) {
        self.map.remove_by_right(client_id);
    }

    #[must_use]
    pub fn get_client_id(&self, user_id: &UserId) -> Option<&ClientId> {
        self.map.get_by_left(user_id)
    }

    #[must_use]
    pub fn get_user_id(&self, client_id: &ClientId) -> Option<&UserId> {
        self.map.get_by_right(client_id)
    }

    #[must_use]
    pub fn get_all_user_ids(&self) -> Vec<&UserId> {
        self.map.left_values().collect()
    }

    #[must_use]
    pub fn get_all_client_ids(&self) -> Vec<&ClientId> {
        self.map.right_values().collect()
    }
}
