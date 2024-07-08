// https://wiki.openttd.org/en/Manual/Orders

use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use shared_util::non_empty_circular_list::NonEmptyCircularList;

use crate::BuildingId;

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum MovementOrderLocation {
    StationId(BuildingId),
}

impl Debug for MovementOrderLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StationId(building_id) => write!(f, "{building_id:?}"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum LoadAction {
    NoLoad,
    LoadAvailable,
    LoadUntilFull,
}

impl Debug for LoadAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoLoad => write!(f, "NL"),
            Self::LoadAvailable => write!(f, "LA"),
            Self::LoadUntilFull => write!(f, "LF"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum UnloadAction {
    NoUnload,
    UnloadAvailable,
    UnloadUntilEmpty,
}

impl Debug for UnloadAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoUnload => write!(f, "NU"),
            Self::UnloadAvailable => write!(f, "UA"),
            Self::UnloadUntilEmpty => write!(f, "UE"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum MovementOrderAction {
    PassingThrough,
    LoadAndUnload(LoadAction, UnloadAction),
}

impl Debug for MovementOrderAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PassingThrough => write!(f, "PT"),
            Self::LoadAndUnload(load_action, unload_action) => {
                write!(f, "{load_action:?}-{unload_action:?}")
            },
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
pub struct MovementOrder {
    pub go_to:  MovementOrderLocation,
    pub action: MovementOrderAction,
}

impl Debug for MovementOrder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}-{:?}", self.go_to, self.action)
    }
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

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct MovementOrders {
    force_stop: bool,
    orders:     NonEmptyCircularList<MovementOrder>,
}

impl Debug for MovementOrders {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.force_stop {
            write!(f, "stop ")?;
        }
        write!(f, "{:?}", self.orders)
    }
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
    pub fn is_force_stopped(&self) -> bool {
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
