use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use shared_physics::projectile::calculate_acceleration;

use crate::client_command::InternalGameCommand;
use crate::game_time::{GameTime, GameTimeDiff};
use crate::military::ProjectileType;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::vector3::Vector3;
use crate::{MilitaryBuildingId, PlayerId, ProjectileId};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
struct ProjectileStaticInfo {
    projectile_id:   ProjectileId,
    owner_id:        PlayerId,
    projectile_type: ProjectileType,
    fired_from:      MilitaryBuildingId,
    fired_at:        GameTime,
    landing_at:      GameTime,
    landing_on:      TileCoordsXZ,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct ProjectileDynamicInfo {
    location: Vector3,
    velocity: Vector3,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct ProjectileInfo {
    static_info:  ProjectileStaticInfo,
    dynamic_info: ProjectileDynamicInfo,
}

impl Debug for ProjectileInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.static_info.projectile_id)
    }
}

impl ProjectileInfo {
    #[must_use]
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        projectile_id: ProjectileId,
        owner_id: PlayerId,
        projectile_type: ProjectileType,
        fired_from: MilitaryBuildingId,
        fired_at: GameTime,
        landing_at: GameTime,
        landing_on: TileCoordsXZ,
        location: Vector3,
        velocity: Vector3,
    ) -> Self {
        Self {
            static_info:  ProjectileStaticInfo {
                projectile_id,
                owner_id,
                projectile_type,
                fired_from,
                fired_at,
                landing_at,
                landing_on,
            },
            dynamic_info: ProjectileDynamicInfo { location, velocity },
        }
    }

    #[must_use]
    pub fn owner_id(&self) -> PlayerId {
        self.static_info.owner_id
    }

    #[must_use]
    pub fn projectile_id(&self) -> ProjectileId {
        self.static_info.projectile_id
    }

    #[must_use]
    pub fn dynamic_info(&self) -> &ProjectileDynamicInfo {
        &self.dynamic_info
    }

    pub fn update_dynamic_info(&mut self, dynamic_info: ProjectileDynamicInfo) {
        self.dynamic_info = dynamic_info;
    }

    #[must_use]
    pub fn fired_from(&self) -> MilitaryBuildingId {
        self.static_info.fired_from
    }

    #[must_use]
    pub fn fired_at(&self) -> GameTime {
        self.static_info.fired_at
    }

    #[must_use]
    pub fn location(&self) -> Vector3 {
        self.dynamic_info.location
    }

    #[must_use]
    pub fn velocity(&self) -> Vector3 {
        self.dynamic_info.velocity
    }

    #[must_use]
    pub(crate) fn generate_commands(
        &self,
        _previous_game_time: GameTime,
        _diff: GameTimeDiff,
        new_game_time: GameTime,
    ) -> Vec<InternalGameCommand> {
        if new_game_time < self.static_info.landing_at {
            vec![]
        } else {
            vec![InternalGameCommand::ProjectileLanded(self.projectile_id())]
        }
    }

    pub fn advance_time_diff(&mut self, time_diff: GameTimeDiff) {
        let delta = time_diff.to_seconds();

        let projectile_properties = self.static_info.projectile_type.projectile_properties();

        // Apply velocity
        self.dynamic_info.location += self.velocity() * delta;

        // Apply total acceleration
        let total_acceleration =
            calculate_acceleration(self.dynamic_info.velocity.into(), &projectile_properties);

        self.dynamic_info.velocity += (total_acceleration * delta).into();
    }
}
