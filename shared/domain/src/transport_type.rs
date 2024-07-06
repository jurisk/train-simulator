use std::ops::{Add, AddAssign};

use serde::{Deserialize, Serialize};

use crate::cargo_map::CargoMap;
use crate::resource_type::ResourceType;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy, Default)]
pub struct CargoAmount(f32);

impl CargoAmount {
    #[must_use]
    pub fn new() -> Self {
        Self(0.0)
    }
}

impl Add for CargoAmount {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum TrainComponentType {
    Engine,
    Car(ResourceType),
}

impl TrainComponentType {
    #[must_use]
    pub fn length_in_tiles(self) -> f32 {
        match self {
            TrainComponentType::Engine => 0.8,
            TrainComponentType::Car(_) => 0.4,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum TransportType {
    Train(Vec<TrainComponentType>),
    RoadVehicle(ResourceType),
    Ship(ResourceType),
}

impl AddAssign for CargoAmount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl TransportType {
    #[must_use]
    pub fn length_in_tiles(&self) -> f32 {
        match self {
            TransportType::Train(components) => {
                components
                    .iter()
                    .map(|component| component.length_in_tiles())
                    .sum()
            },
            TransportType::RoadVehicle(_) => todo!(),
            TransportType::Ship(_) => todo!(),
        }
    }

    #[must_use]
    pub fn cargo_capacity(&self) -> CargoMap {
        let mut result = CargoMap::new();
        match self {
            TransportType::Train(components) => {
                for component in components {
                    match component {
                        TrainComponentType::Engine => {},
                        TrainComponentType::Car(resource_type) => {
                            result.add(*resource_type, CargoAmount(1.0));
                        },
                    }
                }
            },
            TransportType::RoadVehicle(resource_type) => {
                result.add(*resource_type, CargoAmount(0.5));
            },
            TransportType::Ship(resource_type) => {
                result.add(*resource_type, CargoAmount(10.0));
            },
        }
        result
    }
}
