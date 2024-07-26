use std::fmt::{Debug, Formatter};
use std::ops::{Add, AddAssign, Div, Mul, Neg};

use serde::{Deserialize, Serialize};

use crate::cargo_map::CargoMap;
use crate::resource_type::ResourceType;

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Default)]
pub struct CargoAmount(f32);

impl CargoAmount {
    #[must_use]
    pub fn empty() -> Self {
        Self(0.0)
    }

    #[must_use]
    pub fn new(amount: f32) -> Self {
        Self(amount)
    }
}

impl Debug for CargoAmount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl Add for CargoAmount {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Mul<f32> for CargoAmount {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Div for CargoAmount {
    type Output = f32;

    fn div(self, rhs: Self) -> Self::Output {
        self.0 / rhs.0
    }
}

impl Neg for CargoAmount {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum TrainComponentType {
    Engine,
    Car(ResourceType),
}

impl Debug for TrainComponentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TrainComponentType::Engine => write!(f, "E"),
            TrainComponentType::Car(resource_type) => write!(f, "{resource_type:?}"),
        }
    }
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

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum TransportType {
    Train(Vec<TrainComponentType>),
    RoadVehicle(ResourceType),
    Ship(ResourceType),
}

impl Debug for TransportType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportType::Train(components) => {
                write!(f, "T-")?;
                let mut info = vec![];
                for component in components {
                    info.push(format!("{component:?}"));
                }
                write!(f, "{}", info.join("-"))
            },
            TransportType::RoadVehicle(resource_type) => {
                write!(f, "RV-{resource_type:?}")
            },
            TransportType::Ship(resource_type) => {
                write!(f, "S-{resource_type:?}")
            },
        }
    }
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

    // TODO: Make more elegant
    #[must_use]
    pub fn mixed_train() -> Self {
        TransportType::Train(vec![
            TrainComponentType::Engine,
            TrainComponentType::Car(ResourceType::Coal),
            TrainComponentType::Car(ResourceType::Coal),
            TrainComponentType::Car(ResourceType::Coal),
            TrainComponentType::Car(ResourceType::Iron),
            TrainComponentType::Car(ResourceType::Iron),
            TrainComponentType::Car(ResourceType::Iron),
            TrainComponentType::Car(ResourceType::Steel),
            TrainComponentType::Car(ResourceType::Steel),
        ])
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
