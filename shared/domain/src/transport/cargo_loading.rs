use log::debug;
use serde::{Deserialize, Serialize};

use crate::building_state::BuildingState;
use crate::game_time::GameTimeDiff;
use crate::transport::movement_orders::MovementOrderAction;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum CargoLoading {
    NotStarted,
    Unloading { time_left: GameTimeDiff },
    Loading { time_left: GameTimeDiff },
    Finished,
}

impl CargoLoading {
    // TODO HIGH: Actually do loading/unloading, considering the loading/unloading instructions. And the times differ based on that.
    pub fn advance(
        &mut self,
        building_state: &mut BuildingState,
        movement_order_action: MovementOrderAction,
        diff: GameTimeDiff,
    ) -> (GameTimeDiff, bool) {
        debug!(
            "Advancing cargo loading: {:?} {:?} {:?}",
            self, diff, building_state
        );

        match self {
            CargoLoading::NotStarted => {
                *self = CargoLoading::Unloading {
                    time_left: GameTimeDiff::from_seconds(1.0),
                };
                (diff, false)
            },
            CargoLoading::Unloading { time_left } => {
                let time_left = *time_left;
                if time_left <= diff {
                    *self = CargoLoading::Loading {
                        time_left: GameTimeDiff::from_seconds(1.0),
                    };
                    (diff - time_left, false)
                } else {
                    *self = CargoLoading::Unloading {
                        time_left: time_left - diff,
                    };
                    (GameTimeDiff::ZERO, false)
                }
            },
            CargoLoading::Loading { time_left } => {
                let time_left = *time_left;
                if time_left <= diff {
                    *self = CargoLoading::Finished;
                    (diff - time_left, false)
                } else {
                    *self = CargoLoading::Loading {
                        time_left: time_left - diff,
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
