#![allow(clippy::match_same_arms)]

use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::cargo_map::CargoMap;
use crate::resource_type::ResourceType;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::transport_type::CargoAmount;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum ProductionType {
    CoalMine,
    IronMine,
    IronWorks,
    CargoPort,
}

impl Debug for ProductionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductionType::CoalMine => write!(f, "CM"),
            ProductionType::IronMine => write!(f, "IM"),
            ProductionType::IronWorks => write!(f, "IW"),
            ProductionType::CargoPort => write!(f, "CP"),
        }
    }
}

impl ProductionType {
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [
            ProductionType::CoalMine,
            ProductionType::IronMine,
            ProductionType::IronWorks,
            ProductionType::CargoPort,
        ]
    }

    #[must_use]
    pub(crate) fn relative_tiles_used(self) -> TileCoverage {
        TileCoverage::Rectangular {
            north_west_inclusive: TileCoordsXZ::new(-1, -1),
            south_east_inclusive: TileCoordsXZ::new(1, 1),
        }
    }

    #[must_use]
    pub fn transform_per_second(self) -> ResourceTransform {
        const CARGO_PER_SECOND: f32 = 0.1f32;
        match self {
            ProductionType::CoalMine => {
                ResourceTransform::new(
                    vec![],
                    vec![ResourceTransformItem::new(
                        ResourceType::Coal,
                        CargoAmount::new(CARGO_PER_SECOND),
                    )],
                )
            },
            ProductionType::IronMine => {
                ResourceTransform::new(
                    vec![],
                    vec![ResourceTransformItem::new(
                        ResourceType::Iron,
                        CargoAmount::new(CARGO_PER_SECOND),
                    )],
                )
            },
            ProductionType::IronWorks => {
                ResourceTransform::new(
                    vec![
                        ResourceTransformItem::new(
                            ResourceType::Iron,
                            CargoAmount::new(CARGO_PER_SECOND),
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
            ProductionType::CargoPort => {
                ResourceTransform::new(
                    vec![
                        ResourceTransformItem::new(
                            ResourceType::Coal,
                            CargoAmount::new(CARGO_PER_SECOND),
                        ),
                        ResourceTransformItem::new(
                            ResourceType::Iron,
                            CargoAmount::new(CARGO_PER_SECOND),
                        ),
                        ResourceTransformItem::new(
                            ResourceType::Steel,
                            CargoAmount::new(CARGO_PER_SECOND),
                        ),
                    ],
                    vec![],
                )
            },
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

// TODO HIGH: What about money here? Positive and negative? Because the port is strange otherwise.
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
    use crate::cargo_map::CargoMap;
    use crate::production_type::ProductionType;
    use crate::resource_type::ResourceType;
    use crate::transport_type::CargoAmount;

    #[test]
    fn test_coal_mine() {
        let transform = ProductionType::CoalMine.transform_per_second();
        let cargo = CargoMap::new();
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 1.0);
    }

    #[test]
    fn test_iron_works_empty() {
        let transform = ProductionType::IronWorks.transform_per_second();
        let cargo = CargoMap::new();
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 0.0);
    }

    #[test]
    fn test_iron_works_one_component_only_other_empty() {
        let transform = ProductionType::IronWorks.transform_per_second();
        let mut cargo = CargoMap::new();
        cargo.add(ResourceType::Coal, CargoAmount::new(4.0));
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 0.0);
    }

    #[test]
    fn test_iron_abundance() {
        let transform = ProductionType::IronWorks.transform_per_second();
        let mut cargo = CargoMap::new();
        cargo.add(ResourceType::Coal, CargoAmount::new(4.0));
        cargo.add(ResourceType::Iron, CargoAmount::new(4.0));
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 1.0);
    }

    #[test]
    fn test_iron_partial() {
        let transform = ProductionType::IronWorks.transform_per_second();
        let mut cargo = CargoMap::new();
        cargo.add(ResourceType::Coal, CargoAmount::new(0.025));
        cargo.add(ResourceType::Iron, CargoAmount::new(4.0));
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 0.5);
    }
}
