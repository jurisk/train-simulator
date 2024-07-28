use std::collections::HashSet;
use std::fmt::{Debug, Formatter};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::building_info::BuildingInfo;
use crate::cargo_map::CargoMap;
use crate::game_time::GameTimeDiff;
use crate::resource_type::ResourceType;
use crate::transport::movement_orders::MovementOrderAction::UnloadAndLoad;
use crate::transport::movement_orders::{LoadAction, UnloadAction};
use crate::transport::transport_info::TransportInfo;

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum CargoProcessing {
    NotStarted,
    Unloading {
        time_needed: GameTimeDiff,
        time_spent:  GameTimeDiff,
    },
    Loading,
    Finished,
}

impl Debug for CargoProcessing {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CargoProcessing::NotStarted => write!(f, " "),
            CargoProcessing::Unloading {
                time_needed: _,
                time_spent: _,
            } => write!(f, "⏫"),
            CargoProcessing::Loading => write!(f, "⏬"),
            CargoProcessing::Finished => write!(f, "☑"),
        }
    }
}

pub(crate) struct CargoProcessingResult {
    pub new_state: CargoProcessing,
    pub remaining: GameTimeDiff,
}

impl CargoProcessingResult {
    pub fn new(new_state: CargoProcessing, remaining: GameTimeDiff) -> Self {
        Self {
            new_state,
            remaining,
        }
    }

    pub fn advance_to_next_order(&self) -> bool {
        self.new_state == CargoProcessing::NotStarted
    }
}

pub(crate) fn cargo_processing_advance(
    transport_info: &mut TransportInfo,
    building: &mut BuildingInfo,
    resources_accepted_for_unloading: &HashSet<ResourceType>,
    diff: GameTimeDiff,
) -> CargoProcessingResult {
    debug!(
        "Advancing cargo loading: {:?} {:?} {:?}",
        transport_info, building, diff
    );

    let movement_order_action = transport_info
        .dynamic_info
        .movement_orders
        .current_order()
        .action;
    let UnloadAndLoad(unload_action, load_action) = movement_order_action;

    match transport_info.dynamic_info.cargo_loading {
        CargoProcessing::NotStarted => {
            CargoProcessingResult::new(
                CargoProcessing::Unloading {
                    time_needed: GameTimeDiff::from_seconds(1.0),
                    time_spent:  GameTimeDiff::ZERO,
                },
                diff,
            )
        },
        CargoProcessing::Unloading {
            time_needed,
            time_spent,
        } => {
            if unload_action == UnloadAction::NoUnload {
                CargoProcessingResult::new(CargoProcessing::Loading, diff)
            } else {
                let cargo_to_unload = transport_info
                    .cargo_loaded()
                    .filter(|(resource, _)| resources_accepted_for_unloading.contains(&resource));

                // TODO HIGH: Do unloading!
                let time_left = time_needed - time_spent;
                if time_left <= diff {
                    CargoProcessingResult::new(CargoProcessing::Loading, diff - time_left)
                } else {
                    CargoProcessingResult::new(
                        CargoProcessing::Unloading {
                            time_needed,
                            time_spent: time_spent + diff,
                        },
                        GameTimeDiff::ZERO,
                    )
                }
            }
        },
        CargoProcessing::Loading => {
            if load_action == LoadAction::NoLoad {
                CargoProcessingResult::new(CargoProcessing::Finished, diff)
            } else {
                // We will only load the cargo that we are not also unloading, as otherwise we may be unloading and instantly loading the same cargo
                let cargo_to_load: CargoMap = building
                    .shippable_cargo()
                    .filter(|(resource, _)| !resources_accepted_for_unloading.contains(&resource));

                let cargo_to_load =
                    cargo_to_load.cap_at(&transport_info.remaining_cargo_capacity());

                if cargo_to_load == CargoMap::new() {
                    // Nothing to load
                    CargoProcessingResult::new(CargoProcessing::Finished, diff)
                } else {
                    let time_needed = time_for_processing(&cargo_to_load);
                    if time_needed <= diff {
                        // We can load all the cargo
                        building.remove_cargo(&cargo_to_load);
                        transport_info.add_cargo(&cargo_to_load);
                        CargoProcessingResult::new(CargoProcessing::Finished, diff - time_needed)
                    } else {
                        // We can only load some of the cargo
                        let proportion = diff / time_needed;
                        let cargo_to_load = cargo_to_load * proportion;
                        building.remove_cargo(&cargo_to_load);
                        transport_info.add_cargo(&cargo_to_load);
                        CargoProcessingResult::new(CargoProcessing::Loading, GameTimeDiff::ZERO)
                    }
                }
            }
        },
        CargoProcessing::Finished => {
            CargoProcessingResult::new(CargoProcessing::NotStarted, GameTimeDiff::ZERO)
        },
    }
}

const CARGO_PROCESSED_PER_SECOND: f32 = 0.1;
fn time_for_processing(cargo: &CargoMap) -> GameTimeDiff {
    GameTimeDiff::from_seconds(CARGO_PROCESSED_PER_SECOND * cargo.total_amount().as_f32())
}
