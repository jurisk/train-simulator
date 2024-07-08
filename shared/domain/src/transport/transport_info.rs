use std::fmt::{Debug, Formatter};

use log::{debug, error, warn};
use serde::{Deserialize, Serialize};

use crate::building_state::BuildingState;
use crate::cargo_map::CargoMap;
use crate::game_time::GameTimeDiff;
use crate::transport::movement_orders::{MovementOrderLocation, MovementOrders};
use crate::transport::progress_within_tile::ProgressWithinTile;
use crate::transport::track_pathfinding::{find_location_tile_tracks, find_route_to};
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
pub struct TransportDynamicInfo {
    location:        TransportLocation,
    velocity:        TransportVelocity, /* TODO HIGH: Acceleration and deceleration should be gradual */
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
        let route = find_route_to(
            self.dynamic_info.location.tile_path[0],
            current_order.go_to,
            building_state,
        );

        match route {
            None => {
                self.dynamic_info.location.progress_within_tile =
                    ProgressWithinTile::about_to_exit();
                self.dynamic_info.movement_orders.force_stop();
                warn!(
                    "No route found for orders {current_order:?} for transport {:?}, stopping: {self:?}",
                    self.transport_id()
                );
            },
            Some(found) => {
                if found.len() <= 1 {
                    error!(
                        "Found empty route to station for transport {self:?}, this should never happen!",
                    );
                    self.dynamic_info.movement_orders.force_stop();
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

    fn at_location(&self, target: MovementOrderLocation, building_state: &BuildingState) -> bool {
        let current_tile_path = self.dynamic_info.location.tile_path[0];
        find_location_tile_tracks(target, building_state).is_some_and(|targets| {
            targets
                .into_iter()
                .any(|target| target == current_tile_path)
        })
    }

    fn load_and_unload(
        &mut self,
        diff: GameTimeDiff,
        building_state: &mut BuildingState,
    ) -> GameTimeDiff {
        // TODO HIGH: Do it
        diff
    }

    pub fn advance(&mut self, diff: GameTimeDiff, building_state: &mut BuildingState) {
        if self.dynamic_info.movement_orders.is_force_stopped() {
            return;
        }

        let progress_within_tile = self.dynamic_info.location.progress_within_tile();
        if progress_within_tile == ProgressWithinTile::about_to_exit() {
            let current_orders = self.dynamic_info.movement_orders.current_order();
            let at_location = self.at_location(current_orders.go_to, building_state);

            if at_location {
                let remaining = self.load_and_unload(diff, building_state);
                if remaining > GameTimeDiff::ZERO {
                    self.dynamic_info.movement_orders.advance_to_next_order();
                    self.jump_tile(building_state);
                    self.advance_within_tile(remaining, building_state);
                }
            } else {
                self.jump_tile(building_state);
                self.advance_within_tile(diff, building_state);
            }
        } else {
            self.advance_within_tile(diff, building_state);
        }
    }

    fn advance_within_tile(&mut self, diff: GameTimeDiff, building_state: &mut BuildingState) {
        let track_length = self.location().tile_path[0].track_type.length();
        let distance_covered_this_tick = self.velocity() * diff;
        let location = &mut self.dynamic_info.location;
        let distance_remaining_in_tile = track_length
            * (ProgressWithinTile::about_to_exit() - location.progress_within_tile).as_f32();

        if distance_covered_this_tick >= distance_remaining_in_tile {
            // We jump tiles and use the remainder of our time for more actions (recursively)
            location.progress_within_tile = ProgressWithinTile::about_to_exit();
            let time_used = distance_remaining_in_tile / self.velocity();
            self.advance(diff - time_used, building_state);
        } else {
            location.progress_within_tile += distance_covered_this_tick / track_length;
        }
    }
}
