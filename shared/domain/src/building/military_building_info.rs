use std::fmt::{Debug, Formatter};

use log::info;
use serde::{Deserialize, Serialize};

use crate::building::WithRelativeTileCoverage;
use crate::building::building_info::{BuildingInfo, WithCostToBuild, WithOwner, WithTileCoverage};
use crate::building::industry_type::IndustryType;
use crate::building::military_building_type::MilitaryBuildingType;
use crate::cargo_map::CargoMap;
use crate::client_command::InternalGameCommand;
use crate::game_state::GameState;
use crate::game_time::{GameTime, GameTimeDiff};
use crate::military::ProjectileType;
use crate::military::projectile_info::{
    ProjectileDynamicInfo, ProjectileInfo, ProjectileStaticInfo,
};
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::vector3::Vector3;
use crate::{MilitaryBuildingId, PlayerId, ProjectileId};

#[derive(Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct MilitaryBuildingDynamicInfo {
    last_fired_at: GameTime,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct MilitaryBuildingInfo {
    id:                     MilitaryBuildingId,
    owner_id:               PlayerId,
    military_building_type: MilitaryBuildingType,
    reference_tile:         TileCoordsXZ,
    dynamic_info:           MilitaryBuildingDynamicInfo,
}

impl Debug for MilitaryBuildingInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} {:?}",
            self.id(),
            self.reference_tile,
            self.military_building_type,
        )
    }
}

impl MilitaryBuildingInfo {
    #[must_use]
    pub fn new(
        id: MilitaryBuildingId,
        owner_id: PlayerId,
        military_building_type: MilitaryBuildingType,
        reference_tile: TileCoordsXZ,
    ) -> Self {
        Self {
            id,
            owner_id,
            military_building_type,
            reference_tile,
            dynamic_info: MilitaryBuildingDynamicInfo::default(),
        }
    }

    #[must_use]
    pub fn id(&self) -> MilitaryBuildingId {
        self.id
    }

    #[must_use]
    pub fn dynamic_info(&self) -> &MilitaryBuildingDynamicInfo {
        &self.dynamic_info
    }

    pub(crate) fn update_dynamic_info(&mut self, dynamic_info: &MilitaryBuildingDynamicInfo) {
        self.dynamic_info = dynamic_info.clone();
    }

    #[must_use]
    pub fn military_building_type(&self) -> MilitaryBuildingType {
        self.military_building_type
    }

    #[must_use]
    pub fn reference_tile(&self) -> TileCoordsXZ {
        self.reference_tile
    }

    pub(crate) fn update_last_fired_at(&mut self, last_fired_at: GameTime) {
        self.dynamic_info.last_fired_at = self.dynamic_info.last_fired_at.max(last_fired_at);
    }

    #[must_use]
    fn ready_to_fire_at(&self) -> GameTime {
        self.dynamic_info.last_fired_at + self.military_building_type.reload_time()
    }

    #[must_use]
    pub fn generate_commands(
        &self,
        previous_game_time: GameTime,
        _time_diff: GameTimeDiff,
        new_game_time: GameTime,
        game_state: &GameState,
    ) -> Vec<InternalGameCommand> {
        let ready_at = self.ready_to_fire_at();
        if new_game_time >= ready_at {
            // Note: This can miss firing in cases where the reload rate is faster than our time diff tick, and we should have fired multiple times per this tick...
            let fired_at = ready_at.max(previous_game_time);
            let landing_at = fired_at + GameTimeDiff::from_seconds(10.0); // TODO HIGH: Calculate flight time
            let mut location: Vector3 = game_state
                .map_level()
                .terrain()
                .tile_center_coordinate(self.reference_tile())
                .into();
            location.y += 1.0; // This is just for debug purposes
            let landing_on = TileCoordsXZ::new(0, 0); // TODO HIGH: Have a target selection
            // TODO HIGH: For `velocity`, have a targeting mechanism, determine the target location, determine the velocity to hit the target.
            let velocity: Vector3 = Vector3::new(40.0, 60.0, 20.0);
            let projectile_info = ProjectileInfo {
                static_info:  ProjectileStaticInfo {
                    projectile_id: ProjectileId::random(),
                    owner_id: self.owner_id,
                    projectile_type: ProjectileType::Standard,
                    fired_from: self.id,
                    fired_at,
                    landing_at,
                    landing_on,
                },
                dynamic_info: ProjectileDynamicInfo { location, velocity },
            };
            info!("Firing {projectile_info:?}",);
            vec![InternalGameCommand::SpawnProjectile(projectile_info)]
        } else {
            vec![]
        }
    }

    pub fn advance_time_diff(
        &mut self,
        _previous_game_time: GameTime,
        _time_diff: GameTimeDiff,
        _new_game_time: GameTime,
    ) {
        // Empty on purpose, at least for now
    }
}

impl WithRelativeTileCoverage for MilitaryBuildingInfo {
    fn relative_tiles_used(&self) -> TileCoverage {
        self.military_building_type.relative_tiles_used()
    }
}

impl WithTileCoverage for MilitaryBuildingInfo {
    fn covers_tiles(&self) -> TileCoverage {
        self.relative_tiles_used().offset_by(self.reference_tile())
    }
}

impl WithCostToBuild for MilitaryBuildingInfo {
    fn cost_to_build(&self) -> (IndustryType, CargoMap) {
        self.military_building_type.cost_to_build()
    }
}

impl WithOwner for MilitaryBuildingInfo {
    fn owner_id(&self) -> PlayerId {
        self.owner_id
    }
}

impl BuildingInfo for MilitaryBuildingInfo {}
