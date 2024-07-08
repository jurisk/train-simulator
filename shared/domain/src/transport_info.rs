use std::fmt::{Debug, Formatter};

use log::{debug, error, warn};
use serde::{Deserialize, Serialize};

use crate::building_state::BuildingState;
use crate::cargo_map::CargoMap;
use crate::game_time::GameTimeDiff;
use crate::movement_orders::{MovementOrderAction, MovementOrderLocation, MovementOrders};
use crate::track_pathfinding::find_route_to_station;
use crate::transport::progress_within_tile::ProgressWithinTile;
use crate::transport::transport_location::TransportLocation;
use crate::transport::transport_velocity::TransportVelocity;
use crate::transport_type::TransportType;
use crate::{PlayerId, TransportId};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportStaticInfo {
    transport_id:   TransportId,
    owner_id:       PlayerId,
    transport_type: TransportType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportDynamicInfo {
    location:        TransportLocation,
    velocity:        TransportVelocity,
    movement_orders: MovementOrders,
    cargo_loaded:    CargoMap,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportInfo {
    static_info:  TransportStaticInfo,
    dynamic_info: TransportDynamicInfo,
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
    fn velocity(&self) -> TransportVelocity {
        self.dynamic_info.velocity
    }

    #[must_use]
    pub fn transport_type(&self) -> &TransportType {
        &self.static_info.transport_type
    }

    #[must_use]
    fn movement_orders(&self) -> &MovementOrders {
        &self.dynamic_info.movement_orders
    }

    fn jump_tile(&mut self, building_state: &BuildingState) {
        debug!("Jumping tile: {:?}", self);

        let transport_type = self.transport_type().clone();
        let current_order = self.movement_orders().current_order();
        let MovementOrderLocation::StationId(target_station) = current_order.go_to;
        let route = find_route_to_station(
            self.dynamic_info.location.tile_path[0],
            target_station,
            building_state,
        );

        match route {
            None => {
                self.dynamic_info.location.progress_within_tile =
                    ProgressWithinTile::about_to_exit();
                self.dynamic_info.movement_orders.force_stop();
                warn!(
                    "No route found to station {target_station:?} for transport {:?}, stopping: {self:?}",
                    self.transport_id()
                );
            },
            Some(found) => {
                if found.is_empty() {
                    error!(
                        "Found empty route to station for transport {self:?}, this should never happen!",
                    );
                    self.dynamic_info.movement_orders.force_stop();
                } else if found.len() == 1 {
                    // We are at the right station already
                    // TODO HIGH: Implement acceleration/deceleration to stop at the right tile, gradually, like trains do
                    match current_order.action {
                        MovementOrderAction::LoadAndUnload(..) => {
                            // TODO HIGH: Implement station stopping and loading/unloading logic
                            self.dynamic_info.movement_orders.advance_to_next_order();
                            self.jump_tile(building_state);
                            // TODO HIGH: Replace 2 lines above
                        },
                        MovementOrderAction::PassingThrough => {
                            self.dynamic_info.movement_orders.advance_to_next_order();
                            self.jump_tile(building_state);
                        },
                    }
                } else {
                    // The first one is the current tile, so we take the second one
                    let next_tile_track = found[1];
                    self.dynamic_info
                        .location
                        .perform_jump(&transport_type, next_tile_track);
                    debug!("Finished jump: {:?}", self);
                }
            },
        };
    }

    pub fn advance(&mut self, diff: GameTimeDiff, building_state: &BuildingState) {
        if self.dynamic_info.movement_orders.is_force_stopped() {
            return;
        }

        let track_length = self.location().tile_path[0].track_type.length();
        let distance_covered_this_tick = self.velocity() * diff;
        let location = &mut self.dynamic_info.location;
        let distance_remaining_in_tile = track_length
            * (ProgressWithinTile::about_to_exit() - location.progress_within_tile).as_f32();

        if distance_covered_this_tick >= distance_remaining_in_tile {
            // We jump tiles and use the remainder of our time for more actions (recursively)
            self.jump_tile(building_state);
            let time_used = distance_remaining_in_tile / self.velocity();
            self.advance(diff - time_used, building_state);
        } else {
            location.progress_within_tile += distance_covered_this_tick / track_length;
        }

        if distance_covered_this_tick < distance_remaining_in_tile {
            // We set the new progress_in_tile and exit
        } else {
            // We jump tiles and use the remainder of our time for more actions (recursively)
        }
    }
}
