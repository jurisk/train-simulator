use std::fmt::{Debug, Formatter};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::building_state::BuildingState;
use crate::cargo_map::CargoMap;
use crate::game_time::GameTimeDiff;
use crate::transport::movement_orders::MovementOrders;
use crate::transport::transport_location::TransportLocation;
use crate::transport::transport_type::TransportType;
use crate::transport::transport_velocity::TransportVelocity;
use crate::{PlayerId, TransportId};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportStaticInfo {
    transport_id:   TransportId,
    owner_id:       PlayerId,
    transport_type: TransportType,
}

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
        diff: GameTimeDiff,
    ) -> GameTimeDiff {
        debug!(
            "Advancing cargo loading: {:?} {:?} {:?}",
            self, diff, building_state
        );

        match self {
            CargoLoading::NotStarted => {
                *self = CargoLoading::Unloading {
                    time_left: GameTimeDiff::from_seconds(1.0),
                };
                self.advance(building_state, diff)
            },
            CargoLoading::Unloading { time_left } => {
                let time_left = *time_left;
                if time_left <= diff {
                    *self = CargoLoading::Loading {
                        time_left: GameTimeDiff::from_seconds(1.0),
                    };
                    self.advance(building_state, diff - time_left)
                } else {
                    *self = CargoLoading::Unloading {
                        time_left: time_left - diff,
                    };
                    GameTimeDiff::ZERO
                }
            },
            CargoLoading::Loading { time_left } => {
                let time_left = *time_left;
                if time_left <= diff {
                    *self = CargoLoading::Finished;
                    diff - time_left
                } else {
                    *self = CargoLoading::Loading {
                        time_left: time_left - diff,
                    };
                    GameTimeDiff::ZERO
                }
            },
            CargoLoading::Finished => GameTimeDiff::ZERO,
        }
    }
}

// TODO: Make fields `private`?
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportDynamicInfo {
    pub location:        TransportLocation,
    pub velocity:        TransportVelocity, /* TODO HIGH: Acceleration and deceleration should be gradual */
    pub movement_orders: MovementOrders,
    pub cargo_loading:   CargoLoading,
    pub cargo_loaded:    CargoMap,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportInfo {
    pub static_info:  TransportStaticInfo,
    pub dynamic_info: TransportDynamicInfo,
}

impl Debug for TransportInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} {:?} {:?} {:?} {:?}",
            self.static_info.transport_id,
            self.static_info.transport_type,
            self.dynamic_info.location,
            self.dynamic_info.movement_orders,
            self.dynamic_info.velocity,
            self.dynamic_info.cargo_loaded
        )
    }
}

impl TransportInfo {
    #[must_use]
    pub fn new(
        transport_id: TransportId,
        owner_id: PlayerId,
        transport_type: TransportType,
        location: TransportLocation,
        velocity: TransportVelocity,
        movement_orders: MovementOrders,
    ) -> Self {
        Self {
            static_info:  TransportStaticInfo {
                transport_id,
                owner_id,
                transport_type,
            },
            dynamic_info: TransportDynamicInfo {
                location,
                velocity,
                movement_orders,
                cargo_loaded: CargoMap::new(),
                cargo_loading: CargoLoading::NotStarted,
            },
        }
    }

    pub fn update_dynamic_info(&mut self, dynamic_info: &TransportDynamicInfo) {
        self.dynamic_info = dynamic_info.clone();
    }

    #[must_use]
    pub fn dynamic_info(&self) -> TransportDynamicInfo {
        self.dynamic_info.clone()
    }

    #[must_use]
    pub fn owner_id(&self) -> PlayerId {
        self.static_info.owner_id
    }

    #[must_use]
    pub fn transport_id(&self) -> TransportId {
        self.static_info.transport_id
    }

    #[must_use]
    pub fn location(&self) -> &TransportLocation {
        &self.dynamic_info.location
    }

    #[must_use]
    pub fn velocity(&self) -> TransportVelocity {
        self.dynamic_info.velocity
    }

    #[must_use]
    pub fn transport_type(&self) -> &TransportType {
        &self.static_info.transport_type
    }
}
