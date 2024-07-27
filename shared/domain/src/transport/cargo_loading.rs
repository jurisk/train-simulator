use std::fmt::{Debug, Formatter};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::building_info::BuildingInfo;
use crate::game_time::GameTimeDiff;
use crate::transport::movement_orders::MovementOrderAction;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum CargoLoading {
    NotStarted,
    Unloading {
        time_needed: GameTimeDiff,
        time_spent:  GameTimeDiff,
    },
    Loading {
        time_needed: GameTimeDiff,
        time_spent:  GameTimeDiff,
    },
    Finished,
}

impl Debug for CargoLoading {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CargoLoading::NotStarted => write!(f, "∅"),
            CargoLoading::Unloading {
                time_needed,
                time_spent,
            } => write!(f, "▲ {:.0?}%", 100.0 * (*time_spent / *time_needed)),
            CargoLoading::Loading {
                time_needed,
                time_spent,
            } => write!(f, "▼ {:.0?}%", 100.0 * (*time_spent / *time_needed)),
            CargoLoading::Finished => write!(f, "✓"),
        }
    }
}

impl CargoLoading {
    // TODO HIGH: Actually do loading/unloading, considering the loading/unloading instructions. And the times differ based on that.
    pub fn advance(
        &mut self,
        building: &mut BuildingInfo,
        movement_order_action: MovementOrderAction,
        diff: GameTimeDiff,
    ) -> (GameTimeDiff, bool) {
        debug!(
            "Advancing cargo loading: {:?} {:?} {:?}",
            self, diff, building
        );

        match self {
            CargoLoading::NotStarted => {
                *self = CargoLoading::Unloading {
                    time_needed: GameTimeDiff::from_seconds(1.0),
                    time_spent:  GameTimeDiff::ZERO,
                };
                (diff, false)
            },
            CargoLoading::Unloading {
                time_needed,
                time_spent,
            } => {
                let time_left = *time_needed - *time_spent;
                if time_left <= diff {
                    *self = CargoLoading::Loading {
                        time_needed: GameTimeDiff::from_seconds(1.0),
                        time_spent:  GameTimeDiff::ZERO,
                    };
                    (diff - time_left, false)
                } else {
                    *self = CargoLoading::Unloading {
                        time_needed: *time_needed,
                        time_spent:  *time_spent + diff,
                    };
                    (GameTimeDiff::ZERO, false)
                }
            },
            CargoLoading::Loading {
                time_needed,
                time_spent,
            } => {
                let time_left = *time_needed - *time_spent;
                if time_left <= diff {
                    *self = CargoLoading::Finished;
                    (diff - time_left, false)
                } else {
                    *self = CargoLoading::Loading {
                        time_needed: *time_needed,
                        time_spent:  *time_spent + diff,
                    };
                    (GameTimeDiff::ZERO, false)
                }
            },
            CargoLoading::Finished => {
                *self = CargoLoading::NotStarted;
                (GameTimeDiff::ZERO, true)
            },
        }
    }
}
