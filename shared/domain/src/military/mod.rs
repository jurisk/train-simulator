use serde::{Deserialize, Serialize};

pub mod projectile_info;
pub mod projectile_stile;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ProjectileType {
    Standard,
}
