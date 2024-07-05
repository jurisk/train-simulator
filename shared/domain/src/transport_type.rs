use std::collections::HashMap;
use std::ops::{Add, AddAssign};

use serde::{Deserialize, Serialize};

use crate::resource_type::ResourceType;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct CargoAmount(f32);

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
    pub fn cargo_capacity(&self) -> HashMap<ResourceType, CargoAmount> {
        let mut cargo_capacity = HashMap::new();
        match self {
            TransportType::Train(components) => {
                for component in components {
                    match component {
                        TrainComponentType::Engine => {},
                        TrainComponentType::Car(resource_type) => {
                            *cargo_capacity
                                .entry(*resource_type)
                                .or_insert(CargoAmount(0.0)) += CargoAmount(1.0);
                        },
                    }
                }
            },
            TransportType::RoadVehicle(resource_type) => {
                *cargo_capacity
                    .entry(*resource_type)
                    .or_insert(CargoAmount(0.0)) += CargoAmount(0.5);
            },
            TransportType::Ship(resource_type) => {
                *cargo_capacity
                    .entry(*resource_type)
                    .or_insert(CargoAmount(0.0)) += CargoAmount(10.0);
            },
        }
        cargo_capacity
    }
}
