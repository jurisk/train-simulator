use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

/// In a way, it is also a "cargo type"
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum ResourceType {
    Coal,
    Iron,
    Steel,
}

impl Debug for ResourceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ch = match self {
            ResourceType::Coal => 'C',
            ResourceType::Iron => 'I',
            ResourceType::Steel => 'S',
        };
        write!(f, "{ch}")
    }
}

impl ResourceType {
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [ResourceType::Coal, ResourceType::Iron, ResourceType::Steel]
    }
}
