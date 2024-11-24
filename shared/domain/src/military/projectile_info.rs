use serde::{Deserialize, Serialize};

use crate::military::ShellType;
use crate::vector3::Vector3;
use crate::{PlayerId, ProjectileId};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct ProjectileStaticInfo {
    pub projectile_id: ProjectileId,
    pub owner_id:      PlayerId,
    pub shell_type:    ShellType,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct ProjectileDynamicInfo {
    pub location: Vector3,
    pub velocity: Vector3,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct ProjectileInfo {
    pub static_info:  ProjectileStaticInfo,
    pub dynamic_info: ProjectileDynamicInfo,
}

impl ProjectileInfo {
    #[must_use]
    pub fn projectile_id(&self) -> ProjectileId {
        self.static_info.projectile_id
    }

    #[must_use]
    pub fn dynamic_info(&self) -> &ProjectileDynamicInfo {
        &self.dynamic_info
    }
}
