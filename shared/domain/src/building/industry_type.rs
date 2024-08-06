#![allow(clippy::match_same_arms)]

use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::building::CoversTiles;
use crate::cargo_amount::CargoAmount;
use crate::cargo_map::CargoMap;
use crate::resource_type::ResourceType;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum IndustryType {
    CoalMine,
    IronMine,
    IronWorks,
    Warehouse,
}

impl Debug for IndustryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndustryType::CoalMine => write!(f, "CoalMine"),
            IndustryType::IronMine => write!(f, "IronMine"),
            IndustryType::IronWorks => write!(f, "IronWorks"),
            IndustryType::Warehouse => write!(f, "Warehouse"),
        }
    }
}

impl IndustryType {
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [
            IndustryType::CoalMine,
            IndustryType::IronMine,
            IndustryType::IronWorks,
            IndustryType::Warehouse,
        ]
    }

    #[must_use]
    pub fn resources_accepted(self) -> Vec<ResourceType> {
        match self {
            IndustryType::CoalMine => vec![],
            IndustryType::IronMine => vec![],
            IndustryType::IronWorks => vec![ResourceType::Iron, ResourceType::Coal],
            IndustryType::Warehouse => vec![ResourceType::Steel],
        }
    }

    #[must_use]
    pub fn transform_per_second(self) -> ResourceTransform {
        const CARGO_PER_SECOND: f32 = 0.1f32;
        match self {
            IndustryType::CoalMine => {
                ResourceTransform::new(
                    vec![],
                    vec![ResourceTransformItem::new(
                        ResourceType::Coal,
                        CargoAmount::new(CARGO_PER_SECOND),
                    )],
                )
            },
            IndustryType::IronMine => {
                ResourceTransform::new(
                    vec![],
                    vec![ResourceTransformItem::new(
                        ResourceType::Iron,
                        CargoAmount::new(CARGO_PER_SECOND),
                    )],
                )
            },
            IndustryType::IronWorks => {
                // https://marketrealist.com/2015/01/coke-fit-steelmaking-process/
                ResourceTransform::new(
                    vec![
                        ResourceTransformItem::new(
                            ResourceType::Iron,
                            CargoAmount::new(CARGO_PER_SECOND * 2.0),
                        ),
                        ResourceTransformItem::new(
                            ResourceType::Coal,
                            CargoAmount::new(CARGO_PER_SECOND),
                        ),
                    ],
                    vec![ResourceTransformItem::new(
                        ResourceType::Steel,
                        CargoAmount::new(CARGO_PER_SECOND),
                    )],
                )
            },
            IndustryType::Warehouse => {
                ResourceTransform::new(
                    vec![ResourceTransformItem::new(
                        ResourceType::Steel,
                        CargoAmount::ZERO,
                    )],
                    vec![],
                )
            },
        }
    }
}

impl CoversTiles for IndustryType {
    #[must_use]
    fn relative_tiles_used(&self) -> TileCoverage {
        TileCoverage::Rectangular {
            north_west_inclusive: TileCoordsXZ::new(-1, -1),
            south_east_inclusive: TileCoordsXZ::new(1, 1),
        }
    }
}

pub struct ResourceTransformItem {
    pub resource: ResourceType,
    pub amount:   CargoAmount,
}

impl ResourceTransformItem {
    #[must_use]
    pub fn new(resource: ResourceType, amount: CargoAmount) -> Self {
        Self { resource, amount }
    }
}

pub struct ResourceTransform {
    pub inputs:  Vec<ResourceTransformItem>,
    pub outputs: Vec<ResourceTransformItem>,
}

impl ResourceTransform {
    // Later: How do we handle when the stock is too full, and we should stop producing due to that?
    #[must_use]
    pub fn calculate_utilisation_percentage(&self, cargo: &CargoMap, seconds: f32) -> f32 {
        let mut utilisation = 1f32;
        for item in &self.inputs {
            let available = cargo.get(item.resource);
            let required = item.amount * seconds;
            let ratio = available / required;
            utilisation = utilisation.min(ratio);
        }
        utilisation
    }
}

impl ResourceTransform {
    #[must_use]
    pub fn new(inputs: Vec<ResourceTransformItem>, outputs: Vec<ResourceTransformItem>) -> Self {
        Self { inputs, outputs }
    }
}

#[cfg(test)]
mod tests {
    use crate::building::industry_type::IndustryType;
    use crate::cargo_amount::CargoAmount;
    use crate::cargo_map::CargoMap;
    use crate::resource_type::ResourceType;

    #[test]
    fn test_coal_mine() {
        let transform = IndustryType::CoalMine.transform_per_second();
        let cargo = CargoMap::new();
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 1.0);
    }

    #[test]
    fn test_iron_works_empty() {
        let transform = IndustryType::IronWorks.transform_per_second();
        let cargo = CargoMap::new();
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 0.0);
    }

    #[test]
    fn test_iron_works_one_component_only_other_empty() {
        let transform = IndustryType::IronWorks.transform_per_second();
        let mut cargo = CargoMap::new();
        cargo.add(ResourceType::Coal, CargoAmount::new(4.0));
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 0.0);
    }

    #[test]
    fn test_iron_abundance() {
        let transform = IndustryType::IronWorks.transform_per_second();
        let mut cargo = CargoMap::new();
        cargo.add(ResourceType::Coal, CargoAmount::new(4.0));
        cargo.add(ResourceType::Iron, CargoAmount::new(4.0));
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 1.0);
    }

    #[test]
    fn test_iron_partial() {
        let transform = IndustryType::IronWorks.transform_per_second();
        let mut cargo = CargoMap::new();
        cargo.add(ResourceType::Coal, CargoAmount::new(0.025));
        cargo.add(ResourceType::Iron, CargoAmount::new(4.0));
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 0.5);
    }
}
