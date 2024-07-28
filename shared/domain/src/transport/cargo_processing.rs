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
    pub new_state:     CargoProcessing,
    pub remaining:     GameTimeDiff,
    pub cargo_to_load: Option<CargoMap>,
}

impl CargoProcessingResult {
    pub fn new(
        new_state: CargoProcessing,
        remaining: GameTimeDiff,
        cargo_to_load: Option<CargoMap>,
    ) -> Self {
        Self {
            new_state,
            remaining,
            cargo_to_load,
        }
    }

    pub fn advance_to_next_order(&self) -> bool {
        self.new_state == CargoProcessing::NotStarted
    }
}

pub(crate) fn cargo_processing_advance(
    transport_info: &TransportInfo,
    building: &BuildingInfo,
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
                None,
            )
        },
        CargoProcessing::Unloading {
            time_needed,
            time_spent,
        } => {
            if unload_action == UnloadAction::NoUnload {
                CargoProcessingResult::new(CargoProcessing::Loading, diff, None)
            } else {
                let cargo_to_unload = transport_info
                    .cargo_loaded()
                    .filter(|(resource, _)| resources_accepted_for_unloading.contains(&resource));

                // TODO HIGH: Do unloading!
                let time_left = time_needed - time_spent;
                if time_left <= diff {
                    CargoProcessingResult::new(CargoProcessing::Loading, diff - time_left, None)
                } else {
                    CargoProcessingResult::new(
                        CargoProcessing::Unloading {
                            time_needed,
                            time_spent: time_spent + diff,
                        },
                        GameTimeDiff::ZERO,
                        None,
                    )
                }
            }
        },
        CargoProcessing::Loading => {
            if load_action == LoadAction::NoLoad {
                CargoProcessingResult::new(CargoProcessing::Finished, diff, None)
            } else {
                // We will only load the cargo that we are not also unloading, as otherwise we may be unloading and instantly loading the same cargo
                let cargo_to_load: CargoMap = building
                    .shippable_cargo()
                    .filter(|(resource, _)| !resources_accepted_for_unloading.contains(&resource));

                let cargo_to_load =
                    cargo_to_load.cap_at(&transport_info.remaining_cargo_capacity());

                if cargo_to_load == CargoMap::new() {
                    // Nothing to load
                    CargoProcessingResult::new(CargoProcessing::Finished, diff, None)
                } else {
                    let time_needed = time_for_processing(&cargo_to_load);
                    let remaining = diff - time_needed;
                    if remaining <= GameTimeDiff::ZERO {
                        // We can only load some of the cargo
                        let proportion = diff / time_needed;
                        let cargo_to_load = cargo_to_load * proportion;
                        CargoProcessingResult::new(
                            CargoProcessing::Loading,
                            GameTimeDiff::ZERO,
                            Some(cargo_to_load),
                        )
                    } else {
                        // We can load all the cargo
                        CargoProcessingResult::new(
                            CargoProcessing::Finished,
                            remaining,
                            Some(cargo_to_load),
                        )
                    }
                }
            }
        },
        CargoProcessing::Finished => {
            CargoProcessingResult::new(CargoProcessing::NotStarted, GameTimeDiff::ZERO, None)
        },
    }
}

const CARGO_PROCESSED_PER_SECOND: f32 = 0.1;
fn time_for_processing(cargo: &CargoMap) -> GameTimeDiff {
    GameTimeDiff::from_seconds(CARGO_PROCESSED_PER_SECOND * cargo.total_amount().as_f32())
}
