use std::collections::HashSet;
use std::fmt::{Debug, Formatter};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::building::station_info::StationInfo;
use crate::cargo_map::CargoMap;
use crate::game_time::GameTimeDiff;
use crate::resource_type::ResourceType;
use crate::transport::movement_orders::MovementOrderAction::UnloadAndLoad;
use crate::transport::movement_orders::{LoadAction, UnloadAction};
use crate::transport::transport_info::TransportInfo;

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum CargoProcessing {
    NotStarted,
    Unloading,
    Loading,
    Finished,
}

impl Debug for CargoProcessing {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CargoProcessing::NotStarted => write!(f, " "),
            CargoProcessing::Unloading => write!(f, "⏫"),
            CargoProcessing::Loading => write!(f, "⏬"),
            CargoProcessing::Finished => write!(f, "☑"),
        }
    }
}

pub(crate) struct CargoProcessingResult {
    pub new_state:       CargoProcessing,
    pub remaining:       GameTimeDiff,
    pub cargo_to_unload: Option<CargoMap>,
    pub cargo_to_load:   Option<CargoMap>,
}

impl CargoProcessingResult {
    pub fn new(
        new_state: CargoProcessing,
        remaining: GameTimeDiff,
        cargo_to_unload: Option<CargoMap>,
        cargo_to_load: Option<CargoMap>,
    ) -> Self {
        Self {
            new_state,
            remaining,
            cargo_to_unload,
            cargo_to_load,
        }
    }

    pub fn advance_to_next_order(&self) -> bool {
        self.new_state == CargoProcessing::NotStarted
    }
}

pub(crate) fn cargo_processing_advance(
    transport_info: &TransportInfo,
    station: &StationInfo,
    resources_accepted_for_unloading: &HashSet<ResourceType>,
    diff: GameTimeDiff,
) -> CargoProcessingResult {
    debug!(
        "Advancing cargo loading: {:?} {:?} {:?}",
        transport_info, station, diff
    );

    let movement_order_action = transport_info
        .dynamic_info
        .movement_orders
        .current_order()
        .action;
    let UnloadAndLoad(unload_action, load_action) = movement_order_action;

    match transport_info.dynamic_info.cargo_processing {
        CargoProcessing::NotStarted => {
            CargoProcessingResult::new(CargoProcessing::Unloading, diff, None, None)
        },
        CargoProcessing::Unloading => {
            if unload_action == UnloadAction::NoUnload {
                CargoProcessingResult::new(CargoProcessing::Loading, diff, None, None)
            } else {
                let cargo_to_unload = transport_info
                    .cargo_loaded()
                    .filter(|(resource, _)| resources_accepted_for_unloading.contains(&resource));

                let (is_finished, remaining, cargo_to_unload) = time_helper(diff, cargo_to_unload);
                let next_state = if is_finished {
                    CargoProcessing::Loading
                } else {
                    CargoProcessing::Unloading
                };
                CargoProcessingResult::new(next_state, remaining, cargo_to_unload, None)
            }
        },
        CargoProcessing::Loading => {
            if load_action == LoadAction::NoLoad {
                CargoProcessingResult::new(CargoProcessing::Finished, diff, None, None)
            } else {
                // We will only load the cargo that we are not also unloading, as otherwise we may be unloading and instantly loading the same cargo
                let cargo_to_load: CargoMap = station
                    .station_shippable_cargo()
                    .filter(|(resource, _)| !resources_accepted_for_unloading.contains(&resource));

                let cargo_to_load =
                    cargo_to_load.cap_at(&transport_info.remaining_cargo_capacity());

                let (is_finished, remaining, cargo_to_load) = time_helper(diff, cargo_to_load);
                let next_state = if is_finished {
                    CargoProcessing::Finished
                } else {
                    CargoProcessing::Loading
                };
                CargoProcessingResult::new(next_state, remaining, None, cargo_to_load)
            }
        },
        CargoProcessing::Finished => {
            CargoProcessingResult::new(CargoProcessing::NotStarted, GameTimeDiff::ZERO, None, None)
        },
    }
}

fn time_helper(
    diff: GameTimeDiff,
    cargo_to_load: CargoMap,
) -> (bool, GameTimeDiff, Option<CargoMap>) {
    if cargo_to_load == CargoMap::new() {
        // Nothing to load
        (true, diff, None)
    } else {
        let time_needed = time_for_processing(&cargo_to_load);
        let remaining = diff - time_needed;
        if remaining <= GameTimeDiff::ZERO {
            // We can only load some of the cargo
            let proportion = diff / time_needed;
            let cargo_to_load = cargo_to_load * proportion;
            (false, GameTimeDiff::ZERO, Some(cargo_to_load))
        } else {
            // We can load all the cargo
            (true, remaining, Some(cargo_to_load))
        }
    }
}

const CARGO_PROCESSED_PER_SECOND: f32 = 1.0f32;
fn time_for_processing(cargo: &CargoMap) -> GameTimeDiff {
    GameTimeDiff::from_seconds(cargo.total_amount().as_f32() / CARGO_PROCESSED_PER_SECOND)
}
