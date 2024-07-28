use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::{AddAssign, Mul, Neg, Sub, SubAssign};

use serde::{Deserialize, Serialize};

use crate::cargo_amount::CargoAmount;
use crate::resource_type::ResourceType;

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

    pub(crate) fn add(&mut self, resource: ResourceType, amount: CargoAmount) {
        *self.map.entry(resource).or_default() += amount;
    }

    pub(crate) fn get(&self, resource: ResourceType) -> CargoAmount {
        self.map.get(&resource).copied().unwrap_or_default()
    }

    pub(crate) fn total_amount(&self) -> CargoAmount {
        let mut result = CargoAmount::ZERO;
        for &amount in self.map.values() {
            result += amount;
        }
        result
    }

    #[must_use]
    pub fn filter<F>(self, f: F) -> Self
    where
        F: Fn((ResourceType, CargoAmount)) -> bool,
    {
        let map = self
            .map
            .into_iter()
            .filter(|(resource_type, cargo_amount)| f((*resource_type, *cargo_amount)))
            .collect();
        Self { map }
    }

    #[must_use]
    pub fn cap_at(self, cap: &Self) -> Self {
        let map = self
            .map
            .into_iter()
            .map(|(resource_type, cargo_amount)| {
                (resource_type, cargo_amount.min(cap.get(resource_type)))
            })
            .collect();
        Self { map }
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

impl Sub for CargoMap {
    type Output = CargoMap;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = CargoMap::new();
        for (&resource, &amount) in &self.map {
            result.add(resource, amount - rhs.get(resource));
        }
        result
    }
}

impl Mul<f32> for CargoMap {
    type Output = CargoMap;

    fn mul(self, rhs: f32) -> Self::Output {
        let mut result = CargoMap::new();
        for (&resource, &amount) in &self.map {
            result.add(resource, amount * rhs);
        }
        result
    }
}

impl AddAssign<&Self> for CargoMap {
    fn add_assign(&mut self, rhs: &Self) {
        for (resource, amount) in &rhs.map {
            self.add(*resource, *amount);
        }
    }
}

impl SubAssign<&Self> for CargoMap {
    fn sub_assign(&mut self, rhs: &Self) {
        for (resource, amount) in &rhs.map {
            self.add(*resource, -*amount);
        }
    }
}
