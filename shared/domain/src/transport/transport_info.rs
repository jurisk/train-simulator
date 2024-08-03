use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::cargo_amount::CargoAmount;
use crate::cargo_map::CargoMap;
use crate::resource_type::ResourceType;
use crate::transport::cargo_processing::CargoProcessing;
use crate::transport::movement_orders::MovementOrders;
use crate::transport::transport_location::TransportLocation;
use crate::transport::transport_type::TransportType;
use crate::transport::transport_velocity::TransportVelocity;
use crate::{PlayerId, TransportId};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportStaticInfo {
    transport_id:   TransportId,
    owner_id:       PlayerId,
    transport_type: TransportType,
}

// TODO: Make fields `private`?
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportDynamicInfo {
    pub location:         TransportLocation,
    pub velocity:         TransportVelocity, /* TODO: Acceleration and deceleration should be gradual */
    pub movement_orders:  MovementOrders,
    pub cargo_processing: CargoProcessing,
    pub cargo_loaded:     CargoMap,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct TransportInfo {
    pub static_info:  TransportStaticInfo,
    pub dynamic_info: TransportDynamicInfo,
}

impl Debug for TransportInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} {:?} {:?} {:?} {:?}",
            self.static_info.transport_id,
            self.static_info.transport_type,
            self.dynamic_info.location,
            self.dynamic_info.movement_orders,
            self.dynamic_info.velocity,
            self.dynamic_info.cargo_loaded
        )
    }
}

impl TransportInfo {
    #[must_use]
    pub fn new(
        transport_id: TransportId,
        owner_id: PlayerId,
        transport_type: TransportType,
        location: TransportLocation,
        movement_orders: MovementOrders,
    ) -> Self {
        let velocity = transport_type.max_velocity();
        Self {
            static_info:  TransportStaticInfo {
                transport_id,
                owner_id,
                transport_type,
            },
            dynamic_info: TransportDynamicInfo {
                location,
                velocity,
                movement_orders,
                cargo_loaded: CargoMap::new(),
                cargo_processing: CargoProcessing::NotStarted,
            },
        }
    }

    #[must_use]
    pub fn cargo_as_string(&self) -> String {
        let mut results = vec![];
        for resource in ResourceType::all() {
            let amount = self.dynamic_info.cargo_loaded.get(resource);
            let capacity = self
                .static_info
                .transport_type
                .cargo_capacity()
                .get(resource);
            if capacity != CargoAmount::ZERO {
                let amount_string = if amount == CargoAmount::ZERO {
                    "―".to_string()
                } else {
                    format!("{amount:.2?}")
                };
                let as_string = format!("{resource:?} {amount_string}/{capacity:.2?}");
                results.push(as_string);
            }
        }
        results.join("  ")
    }

    #[must_use]
    pub fn cargo_loaded(&self) -> CargoMap {
        self.dynamic_info.cargo_loaded.clone()
    }

    #[must_use]
    pub fn cargo_processing(&self) -> CargoProcessing {
        self.dynamic_info.cargo_processing
    }

    #[must_use]
    pub fn remaining_cargo_capacity(&self) -> CargoMap {
        self.static_info.transport_type.cargo_capacity() - self.dynamic_info.cargo_loaded.clone()
    }

    pub fn update_dynamic_info(&mut self, dynamic_info: &TransportDynamicInfo) {
        self.dynamic_info = dynamic_info.clone();
    }

    #[must_use]
    pub fn movement_orders(&self) -> &MovementOrders {
        &self.dynamic_info.movement_orders
    }

    #[must_use]
    pub fn dynamic_info(&self) -> TransportDynamicInfo {
        self.dynamic_info.clone()
    }

    #[must_use]
    pub fn owner_id(&self) -> PlayerId {
        self.static_info.owner_id
    }

    #[must_use]
    pub fn transport_id(&self) -> TransportId {
        self.static_info.transport_id
    }

    #[must_use]
    pub fn location(&self) -> &TransportLocation {
        &self.dynamic_info.location
    }

    #[must_use]
    pub fn velocity(&self) -> TransportVelocity {
        self.dynamic_info.velocity
    }

    #[must_use]
    pub fn transport_type(&self) -> &TransportType {
        &self.static_info.transport_type
    }

    pub fn add_cargo(&mut self, cargo: &CargoMap) {
        self.dynamic_info.cargo_loaded += cargo;
    }

    pub fn remove_cargo(&mut self, cargo: &CargoMap) {
        self.dynamic_info.cargo_loaded -= cargo;
    }
}
