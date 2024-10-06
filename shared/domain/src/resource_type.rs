use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::resource_type::ResourceType::{
    Ammunition, ArtilleryWeapons, Cellulose, Cement, Clay, Coal, Concrete, Explosives,
    FarmProducts, Food, Fuel, Iron, Limestone, Nitrates, Oil, SandAndGravel, Steel, Sulfur, Timber,
    Wood,
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
    Explosives,
    Food,
    Fuel,
    Steel,
    Timber,
    ArtilleryWeapons,
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
            Explosives,
            Food,
            Fuel,
            Steel,
            Timber,
            ArtilleryWeapons,
        ]
    }
}
