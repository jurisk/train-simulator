use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use log::warn;
use serde::{Deserialize, Serialize};

use crate::client_command::InternalGameCommand;
use crate::game_time::{GameTime, GameTimeDiff};
use crate::military::projectile_info::{ProjectileDynamicInfo, ProjectileInfo};
use crate::{PlayerId, ProjectileId};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct ProjectileState {
    projectiles: HashMap<ProjectileId, ProjectileInfo>,
}

impl ProjectileState {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            projectiles: HashMap::new(),
        }
    }

    #[must_use]
    pub fn all_projectiles(&self) -> Vec<&ProjectileInfo> {
        self.projectiles.values().collect()
    }

    #[must_use]
    pub fn find_projectile(&self, projectile_id: ProjectileId) -> Option<&ProjectileInfo> {
        self.projectiles.get(&projectile_id)
    }

    #[must_use]
    pub fn find_projectiles_by_owner(
        &self,
        owner_id: PlayerId,
    ) -> impl IntoIterator<Item = &ProjectileInfo> {
        self.projectiles
            .values()
            .filter(move |projectile_info| projectile_info.static_info.owner_id == owner_id)
    }

    pub(crate) fn upsert(&mut self, projectile: ProjectileInfo) {
        let projectile_id = projectile.projectile_id();
        self.projectiles.insert(projectile_id, projectile);
    }

    pub(crate) fn remove(&mut self, projectile_id: ProjectileId) {
        self.projectiles.remove(&projectile_id);
    }

    pub fn update_dynamic_infos(
        &mut self,
        projectile_dynamic_infos: &HashMap<ProjectileId, ProjectileDynamicInfo>,
    ) {
        for (projectile_id, dynamic_info) in projectile_dynamic_infos {
            if let Some(projectile_info) = self.projectiles.get_mut(projectile_id) {
                projectile_info.dynamic_info = dynamic_info.clone();
            } else {
                warn!("No projectile found for dynamic info: {projectile_id:?}");
            }
        }
    }

    pub(crate) fn generate_commands(
        &self,
        previous_game_time: GameTime,
        diff: GameTimeDiff,
        new_game_time: GameTime,
    ) -> Vec<InternalGameCommand> {
        self.projectiles
            .values()
            .flat_map(|projectile_info| {
                projectile_info.generate_commands(previous_game_time, diff, new_game_time)
            })
            .collect()
    }

    pub fn advance_time_diff(&mut self, time_diff: GameTimeDiff) {
        for projectile_info in self.projectiles.values_mut() {
            projectile_info.advance_time_diff(time_diff);
        }
    }
}

impl Debug for ProjectileState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProjectileState({} projectiles)", self.projectiles.len())
    }
}
