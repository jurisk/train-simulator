use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum ResourceType {
    Coal,
    Iron,
    Steel,
}

impl ResourceType {
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [ResourceType::Coal, ResourceType::Iron, ResourceType::Steel]
    }
}
