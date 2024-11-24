use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use log::warn;
use serde::{Deserialize, Serialize};

use crate::ProjectileId;
use crate::military::projectile_info::{ProjectileDynamicInfo, ProjectileInfo};

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
}

impl Debug for ProjectileState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProjectileState({} projectiles)", self.projectiles.len())
    }
}
