use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};

use bevy_math::Vec3;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use shared_util::direction_xz::DirectionXZ;
use shared_util::non_empty_circular_list::NonEmptyCircularList;

use crate::building_state::BuildingState;
use crate::game_time::GameTimeDiff;
use crate::tile_track::TileTrack;
use crate::track_pathfinding::find_route_to_station;
use crate::transport_type::TransportType;
use crate::{BuildingId, PlayerId, TransportId};

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct ProgressWithinTile(f32);

impl Debug for ProgressWithinTile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl Eq for ProgressWithinTile {}

impl PartialOrd for ProgressWithinTile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProgressWithinTile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or_else(|| panic!("Failed to compare {self:?} and {other:?}"))
    }
}

impl ProgressWithinTile {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(progress: f32) -> Self {
        assert!(
            (0.0 ..= 1.0).contains(&progress),
            "Progress must be between 0.0 and 1.0, but was {progress}"
        );
        Self(progress)
    }

    #[must_use]
    pub fn from_point_between_two_points(start_end: (Vec3, Vec3), point: Vec3) -> Self {
        let (start, end) = start_end;
        let value = (point - start).length() / (end - start).length();
        Self::new(value)
    }

    #[must_use]
    pub fn just_entering() -> Self {
        Self(0.0)
    }

    #[must_use]
    pub fn about_to_exit() -> Self {
        Self(1.0)
    }

    #[must_use]
    pub fn out_of_bounds(self) -> bool {
        self.progress() >= 1.0
    }

    #[must_use]
    pub fn progress(self) -> f32 {
        self.0
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportLocation {
    pub tile_path:            Vec<TileTrack>, /* Which tile is it on now, and which tiles has it been on - only as much as to cover the vehicle's length */
    pub progress_within_tile: ProgressWithinTile,
}

impl TransportLocation {
    #[must_use]
    pub fn entering_from(&self) -> DirectionXZ {
        let current_tile_track = self.tile_path[0];
        current_tile_track
            .track_type
            .other_end(current_tile_track.pointing_in)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct TransportVelocity {
    pub tiles_per_second: f32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct MovementOrders {
    is_stopped: bool,
    stations:   NonEmptyCircularList<BuildingId>,
}

impl MovementOrders {
    #[must_use]
    pub fn one(station_id: BuildingId) -> Self {
        Self {
            is_stopped: false,
            stations:   NonEmptyCircularList::one(station_id),
        }
    }

    pub fn push(&mut self, station_id: BuildingId) {
        self.stations.push(station_id);
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportStaticInfo {
    pub transport_id:   TransportId,
    pub owner_id:       PlayerId,
    pub transport_type: TransportType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportDynamicInfo {
    pub location:        TransportLocation,
    pub velocity:        TransportVelocity,
    pub movement_orders: MovementOrders,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportInfo {
    static_info:  TransportStaticInfo,
    dynamic_info: TransportDynamicInfo,
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
            },
        }
    }

    pub fn update_dynamic_info(&mut self, dynamic_info: &TransportDynamicInfo) {
        self.dynamic_info = dynamic_info.clone();
    }

    #[must_use]
    pub fn id(&self) -> TransportId {
        self.static_info.transport_id
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

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::items_after_statements
    )]
    fn jump_tile(&mut self, building_state: &BuildingState) {
        info!("Jumping tile: {:?}", self);

        let transport_type = self.transport_type().clone();
        let id = self.id();

        let movement_orders = self.movement_orders().clone();
        let location = &mut self.dynamic_info.location;
        let current_tile_track = location.tile_path[0];

        let target_station = movement_orders.stations.next();
        let route = find_route_to_station(current_tile_track, target_station, building_state);
        let next = match route {
            None => None,
            Some(found) => found.get(1).copied(), // The first one is the current tile
        };
        match next {
            None => {
                location.progress_within_tile = ProgressWithinTile::about_to_exit();
                self.dynamic_info.movement_orders.is_stopped = true;
                warn!(
                    "No route found to station {target_station:?} for transport {id:?}, stopping: {self:?}",
                );
            },
            Some(next_tile_track) => {
                location.tile_path.insert(0, next_tile_track);

                // Later: We are rather crudely sometimes removing the last element when we are inserting an
                // element.
                // This means - depending on `HEURISTIC_COEF` - that sometimes we will be carrying around
                // "too many tiles", or it could lead to running out of tiles if it is too short.
                // The alternative is to use `calculate_train_component_head_tails_and_final_tail_position`
                // to calculate the tail position, and then remove the last tiles if they are not needed,
                // but that introduces more complexity.
                const HEURISTIC_COEF: f32 = 2.0;
                if location.tile_path.len()
                    > (HEURISTIC_COEF * transport_type.length_in_tiles()) as usize
                {
                    let _ = location.tile_path.pop();
                }

                location.progress_within_tile.0 -= 1.0;

                info!("Finished jump: {:?}", self);
            },
        }
    }

    fn normalise_progress_jumping_tiles(&mut self, building_state: &BuildingState) {
        while self.location().progress_within_tile.out_of_bounds()
            && !self.dynamic_info.movement_orders.is_stopped
        {
            self.jump_tile(building_state);
        }
    }

    pub fn advance(&mut self, diff: GameTimeDiff, building_state: &BuildingState) {
        if self.dynamic_info.movement_orders.is_stopped {
            return;
        }

        let seconds = diff.to_seconds();
        let TransportVelocity { tiles_per_second } = self.velocity();
        let track_type = self.location().tile_path[0].track_type;
        let location = &mut self.dynamic_info.location;
        let track_length = track_type.length_in_tiles();
        let ProgressWithinTile(progress_within_tile) = location.progress_within_tile;
        let effective_speed = tiles_per_second / track_length;
        let new_progress = progress_within_tile + effective_speed * seconds;
        let new_progress = ProgressWithinTile(new_progress);
        location.progress_within_tile = new_progress;
        self.normalise_progress_jumping_tiles(building_state);
    }
}
