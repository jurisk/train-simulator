use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::building::building_info::WithCostToBuild;
use crate::building::industry_type::IndustryType;
use crate::cargo_amount::CargoAmount;
use crate::cargo_map::CargoMap;
use crate::resource_type::ResourceType;
use crate::transport::transport_velocity::TransportVelocity;

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
    pub fn cargo_train(resource_type: ResourceType) -> Self {
        TransportType::Train(vec![
            TrainComponentType::Engine,
            TrainComponentType::Car(resource_type),
            TrainComponentType::Car(resource_type),
            TrainComponentType::Car(resource_type),
            TrainComponentType::Car(resource_type),
            TrainComponentType::Car(resource_type),
            TrainComponentType::Car(resource_type),
            TrainComponentType::Car(resource_type),
            TrainComponentType::Car(resource_type),
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
                            result.add(*resource_type, CargoAmount::new(1.0));
                        },
                    }
                }
            },
            TransportType::RoadVehicle(resource_type) => {
                result.add(*resource_type, CargoAmount::new(0.5));
            },
            TransportType::Ship(resource_type) => {
                result.add(*resource_type, CargoAmount::new(10.0));
            },
        }
        result
    }

    #[must_use]
    pub fn max_velocity(&self) -> TransportVelocity {
        TransportVelocity::new(2.0)
    }
}

impl WithCostToBuild for TransportType {
    fn cost_to_build(&self) -> (IndustryType, CargoMap) {
        (
            IndustryType::ConstructionYard,
            CargoMap::single(ResourceType::Steel, 1.0),
        )
    }
}
