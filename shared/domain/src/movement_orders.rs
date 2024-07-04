// https://wiki.openttd.org/en/Manual/Orders

use serde::{Deserialize, Serialize};
use shared_util::non_empty_circular_list::NonEmptyCircularList;

use crate::BuildingId;

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum MovementOrderLocation {
    StationId(BuildingId),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum LoadAction {
    NoLoad,
    LoadAvailable,
    LoadUntilFull,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum UnloadAction {
    NoUnload,
    UnloadAvailable,
    UnloadUntilEmpty,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum MovementOrderAction {
    PassingThrough,
    LoadAndUnload(LoadAction, UnloadAction),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Copy, Clone)]
pub struct MovementOrder {
    pub go_to:  MovementOrderLocation,
    pub action: MovementOrderAction,
}

impl MovementOrder {
    #[must_use]
    pub fn stop_at_station(building_id: BuildingId) -> Self {
        Self {
            go_to:  MovementOrderLocation::StationId(building_id),
            action: MovementOrderAction::LoadAndUnload(
                LoadAction::LoadAvailable,
                UnloadAction::UnloadAvailable,
            ),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct MovementOrders {
    force_stop: bool,
    orders:     NonEmptyCircularList<MovementOrder>,
}

impl MovementOrders {
    #[must_use]
    pub fn one(movement_order: MovementOrder) -> Self {
        Self {
            force_stop: false,
            orders:     NonEmptyCircularList::one(movement_order),
        }
    }

    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.force_stop
    }

    pub fn force_stop(&mut self) {
        self.force_stop = true;
    }

    pub fn push(&mut self, movement_order: MovementOrder) {
        self.orders.push(movement_order);
    }

    #[must_use]
    pub fn current_order(&self) -> MovementOrder {
        self.orders.next()
    }

    pub fn advance_to_next_order(&mut self) {
        self.orders.advance();
    }
}
