use log::info;
use serde::{Deserialize, Serialize};

use crate::game_time::{GameTime, GameTimeDiff};
use crate::military::ProjectileType;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::vector3::Vector3;
use crate::{MilitaryBuildingId, PlayerId, ProjectileId};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ProjectileStaticInfo {
    pub projectile_id:   ProjectileId,
    pub owner_id:        PlayerId,
    pub projectile_type: ProjectileType,
    pub fired_from:      MilitaryBuildingId,
    pub fired_at:        GameTime,
    pub landing_at:      GameTime,
    pub landing_on:      TileCoordsXZ,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ProjectileDynamicInfo {
    pub location: Vector3,
    pub velocity: Vector3,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
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

    pub fn advance_time_diff(&mut self, time_diff: GameTimeDiff) {
        // TODO HIGH: Take physics model from 'kido-butai', including drag.
        let gravity = Vector3::new(0.0, -9.81, 0.0);
        self.dynamic_info.velocity += gravity * time_diff.to_seconds();
        self.dynamic_info.location += self.dynamic_info.velocity * time_diff.to_seconds();
        info!(
            "Projectile {projectile_id:?} advanced to {location:?}",
            projectile_id = self.projectile_id(),
            location = self.dynamic_info.location
        );
    }
}
