use serde::{Deserialize, Serialize};

use crate::cargo_map::CargoMap;
use crate::resource_type::ResourceType;

pub mod projectile_info;
pub mod projectile_state;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ProjectileType {
    Standard,
}

impl ProjectileType {
    #[must_use]
    pub fn cost_per_shot(&self) -> CargoMap {
        match self {
            ProjectileType::Standard => CargoMap::single(ResourceType::Ammunition, 0.01),
        }
    }
}
