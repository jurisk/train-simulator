use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::resource_type::ResourceType::{
    Ammunition, Cellulose, Cement, Clay, Coal, Concrete, FarmProducts, Food, Fuel, Iron, Limestone,
    Nitrates, Nitrocellulose, Oil, SandAndGravel, Steel, Sulfur, Timber, Weapons, Wood,
};

/// In a way, it is also a "cargo type"
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub enum ResourceType {
    // Raw
    Clay,
    Coal,
    FarmProducts,
    Iron,
    Limestone,
    Nitrates,
    Oil,
    SandAndGravel,
    Sulfur,
    Wood,
    // Derived
    Ammunition,
    Cellulose,
    Cement,
    Concrete,
    Food,
    Fuel,
    Nitrocellulose,
    Steel,
    Timber,
    Weapons,
}

impl ResourceType {
    #[must_use]
    pub const fn all() -> [Self; 20] {
        [
            // Raw
            Clay,
            Coal,
            FarmProducts,
            Iron,
            Limestone,
            Nitrates,
            Oil,
            SandAndGravel,
            Sulfur,
            Wood,
            // Derived
            Ammunition,
            Cellulose,
            Cement,
            Concrete,
            Food,
            Fuel,
            Nitrocellulose,
            Steel,
            Timber,
            Weapons,
        ]
    }
}
