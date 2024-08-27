#![allow(clippy::into_iter_without_iter)]

// https://wiki.openttd.org/en/Manual/Orders

use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use shared_util::non_empty_circular_list::{NonEmptyCircularList, NonEmptyCircularListIterator};

use crate::StationId;

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum MovementOrderLocation {
    Station(StationId),
}

impl Debug for MovementOrderLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Station(station_id) => write!(f, "{station_id:?}"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum LoadAction {
    NoLoad,
    Load,
}

impl Debug for LoadAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoLoad => write!(f, "No Load"),
            Self::Load => write!(f, "Load"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum UnloadAction {
    NoUnload,
    Unload,
}

impl Debug for UnloadAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoUnload => write!(f, "No Unload"),
            Self::Unload => write!(f, "Unload"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum MovementOrderAction {
    UnloadAndLoad(UnloadAction, LoadAction),
}

impl Debug for MovementOrderAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnloadAndLoad(unload_action, load_action) => {
                write!(f, "{unload_action:?}-{load_action:?}")
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
    pub fn stop_at_station(station_id: StationId) -> Self {
        Self {
            go_to:  MovementOrderLocation::Station(station_id),
            action: MovementOrderAction::UnloadAndLoad(UnloadAction::Unload, LoadAction::Load),
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

    pub fn set_force_stop(&mut self, value: bool) {
        self.force_stop = value;
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

    #[must_use]
    pub fn next_index(&self) -> usize {
        self.orders.next_index()
    }

    pub fn remove_by_index(&mut self, index: usize) {
        self.orders.remove_by_index(index);
    }

    #[must_use]
    pub fn contains_station(&self, station_id: StationId) -> bool {
        self.orders.iter().any(|order| {
            let MovementOrderLocation::Station(order_station_id) = order.go_to;
            order_station_id == station_id
        })
    }
}

pub struct MovementOrdersIterator<'a> {
    orders_iter: NonEmptyCircularListIterator<'a, MovementOrder>,
}

impl<'a> Iterator for MovementOrdersIterator<'a> {
    type Item = &'a MovementOrder;

    fn next(&mut self) -> Option<Self::Item> {
        self.orders_iter.next()
    }
}

impl<'a> IntoIterator for &'a MovementOrders {
    type IntoIter = MovementOrdersIterator<'a>;
    type Item = &'a MovementOrder;

    fn into_iter(self) -> Self::IntoIter {
        MovementOrdersIterator {
            orders_iter: self.orders.into_iter(),
        }
    }
}
