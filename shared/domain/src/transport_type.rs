use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum TrainComponentType {
    Engine,
    Car,
}

impl TrainComponentType {
    #[must_use]
    pub fn length_in_tiles(self) -> f32 {
        match self {
            TrainComponentType::Engine => 0.8,
            TrainComponentType::Car => 0.4,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum TransportType {
    Train(Vec<TrainComponentType>),
    RoadVehicle,
    Ship,
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
            TransportType::RoadVehicle => todo!(),
            TransportType::Ship => todo!(),
        }
    }
}
