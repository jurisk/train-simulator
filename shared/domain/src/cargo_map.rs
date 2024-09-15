use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::ops::{AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use serde::{Deserialize, Serialize};

use crate::cargo_amount::CargoAmount;
use crate::resource_type::ResourceType;

pub trait WithCargo {
    fn cargo(&self) -> &CargoMap;
}

pub trait WithCargoMut {
    fn cargo_mut(&mut self) -> &mut CargoMap;
}

pub trait CargoOps {
    fn add_cargo(&mut self, cargo: &CargoMap);
    fn remove_cargo(&mut self, cargo: &CargoMap);
}

impl<T: WithCargoMut> CargoOps for T {
    fn add_cargo(&mut self, cargo: &CargoMap) {
        *self.cargo_mut() += cargo;
    }

    fn remove_cargo(&mut self, cargo: &CargoMap) {
        *self.cargo_mut() -= cargo;
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct CargoMap {
    map: HashMap<ResourceType, CargoAmount>,
}

impl Default for CargoMap {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> From<[(ResourceType, CargoAmount); N]> for CargoMap {
    fn from(arr: [(ResourceType, CargoAmount); N]) -> Self {
        Self {
            map: HashMap::from(arr),
        }
    }
}

impl<const N: usize> From<[(ResourceType, f32); N]> for CargoMap {
    fn from(arr: [(ResourceType, f32); N]) -> Self {
        let arr = arr.map(|(resource, amount)| (resource, CargoAmount::new(amount)));
        Self::from(arr)
    }
}

impl CargoMap {
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    #[must_use]
    pub fn single(resource: ResourceType, amount: f32) -> Self {
        let mut map = HashMap::new();
        map.insert(resource, CargoAmount::new(amount));
        Self { map }
    }

    pub(crate) fn add(&mut self, resource: ResourceType, amount: CargoAmount) {
        *self.map.entry(resource).or_default() += amount;
    }

    #[must_use]
    pub fn get(&self, resource: ResourceType) -> CargoAmount {
        self.map.get(&resource).copied().unwrap_or_default()
    }

    #[must_use]
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

    #[must_use]
    pub fn contains_resource(&self, resource: ResourceType) -> bool {
        self.get(resource) != CargoAmount::ZERO
    }

    #[must_use]
    pub fn resource_types_present(&self) -> HashSet<ResourceType> {
        let mut result = HashSet::new();
        for resource in ResourceType::all() {
            if self.contains_resource(resource) {
                result.insert(resource);
            }
        }
        result
    }

    #[must_use]
    pub fn is_superset_of(&self, other: &Self) -> bool {
        for (resource, amount) in &other.map {
            if self.get(*resource) < *amount {
                return false;
            }
        }
        true
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
            write!(f, " ")
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

impl AddAssign<&CargoMap> for &mut CargoMap {
    fn add_assign(&mut self, rhs: &CargoMap) {
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

impl SubAssign<&CargoMap> for &mut CargoMap {
    fn sub_assign(&mut self, rhs: &CargoMap) {
        for (resource, amount) in &rhs.map {
            self.add(*resource, -*amount);
        }
    }
}

impl MulAssign<f32> for CargoMap {
    fn mul_assign(&mut self, rhs: f32) {
        for amount in self.map.values_mut() {
            *amount *= rhs;
        }
    }
}
