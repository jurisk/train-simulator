use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Neg;

use serde::{Deserialize, Serialize};

use crate::resource_type::ResourceType;
use crate::transport::transport_type::CargoAmount;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
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

    pub(crate) fn add_all(&mut self, other: &Self) {
        for (resource, amount) in &other.map {
            *self.map.entry(*resource).or_default() += *amount;
        }
    }

    pub(crate) fn add(&mut self, resource: ResourceType, amount: CargoAmount) {
        *self.map.entry(resource).or_default() += amount;
    }

    pub(crate) fn get(&self, resource: ResourceType) -> CargoAmount {
        self.map.get(&resource).copied().unwrap_or_default()
    }
}

impl Debug for CargoMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut results = vec![];
        for resource in ResourceType::all() {
            let amount = self.get(resource);
            if amount != CargoAmount::ZERO {
                let as_string = format!("{resource:?} {amount:?}");
                results.push(as_string);
            }
        }
        if results.is_empty() {
            write!(f, "Empty")
        } else {
            write!(f, "{}", results.join(", "))
        }
    }
}

impl Neg for CargoMap {
    type Output = CargoMap;

    fn neg(self) -> Self::Output {
        let mut result = CargoMap::new();
        for (&resource, &amount) in &self.map {
            result.add(resource, -amount);
        }
        result
    }
}
