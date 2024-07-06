use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::resource_type::ResourceType;
use crate::transport_type::CargoAmount;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CargoMap {
    map: HashMap<ResourceType, CargoAmount>,
}

impl CargoMap {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub(crate) fn add(&mut self, resource: ResourceType, amount: CargoAmount) {
        *self.map.entry(resource).or_default() += amount;
    }
}
