#![allow(clippy::match_same_arms)]

use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::building::WithRelativeTileCoverage;
use crate::cargo_amount::CargoAmount;
use crate::cargo_map::CargoMap;
use crate::map_level::zoning::ZoningType;
use crate::resource_type::ResourceType;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum IndustryType {
    CoalMine,
    IronMine,
    SteelMill,
    Warehouse,
}

impl Debug for IndustryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndustryType::CoalMine => write!(f, "CoalMine"),
            IndustryType::IronMine => write!(f, "IronMine"),
            IndustryType::SteelMill => write!(f, "SteelMill"),
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
            IndustryType::SteelMill,
            IndustryType::Warehouse,
        ]
    }

    #[must_use]
    pub fn required_zoning(self) -> ZoningType {
        match self {
            IndustryType::CoalMine => ZoningType::Deposit(ResourceType::Coal),
            IndustryType::IronMine => ZoningType::Deposit(ResourceType::Iron),
            IndustryType::SteelMill => ZoningType::Industrial,
            IndustryType::Warehouse => ZoningType::Industrial,
        }
    }

    #[must_use]
    pub fn resources_accepted(self) -> Vec<ResourceType> {
        self.transform_per_second()
            .inputs
            .iter()
            .map(|item| item.resource)
            .collect()
    }

    #[must_use]
    pub fn transform_per_second(self) -> ResourceTransform {
        match self {
            IndustryType::CoalMine => {
                ResourceTransform::make(vec![], vec![(ResourceType::Coal, 1.0)])
            },
            IndustryType::IronMine => {
                ResourceTransform::make(vec![], vec![(ResourceType::Iron, 1.0)])
            },
            IndustryType::SteelMill => {
                // https://marketrealist.com/2015/01/coke-fit-steelmaking-process/
                ResourceTransform::make(
                    vec![(ResourceType::Iron, 2.0), (ResourceType::Coal, 1.0)],
                    vec![(ResourceType::Steel, 1.0)],
                )
            },
            IndustryType::Warehouse => {
                ResourceTransform::make(vec![(ResourceType::Steel, 0.0)], vec![])
            },
        }
    }
}

impl WithRelativeTileCoverage for IndustryType {
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
    const CARGO_PER_SECOND: f32 = 0.1f32;

    #[must_use]
    pub fn make(inputs: Vec<(ResourceType, f32)>, outputs: Vec<(ResourceType, f32)>) -> Self {
        ResourceTransform::new(
            inputs
                .into_iter()
                .map(|(resource, coef)| {
                    ResourceTransformItem::new(
                        resource,
                        CargoAmount::new(Self::CARGO_PER_SECOND * coef),
                    )
                })
                .collect(),
            outputs
                .into_iter()
                .map(|(resource, coef)| {
                    ResourceTransformItem::new(
                        resource,
                        CargoAmount::new(Self::CARGO_PER_SECOND * coef),
                    )
                })
                .collect(),
        )
    }

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
        let transform = IndustryType::SteelMill.transform_per_second();
        let cargo = CargoMap::new();
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 0.0);
    }

    #[test]
    fn test_iron_works_one_component_only_other_empty() {
        let transform = IndustryType::SteelMill.transform_per_second();
        let mut cargo = CargoMap::new();
        cargo.add(ResourceType::Coal, CargoAmount::new(4.0));
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 0.0);
    }

    #[test]
    fn test_iron_abundance() {
        let transform = IndustryType::SteelMill.transform_per_second();
        let mut cargo = CargoMap::new();
        cargo.add(ResourceType::Coal, CargoAmount::new(4.0));
        cargo.add(ResourceType::Iron, CargoAmount::new(4.0));
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 1.0);
    }

    #[test]
    fn test_iron_partial() {
        let transform = IndustryType::SteelMill.transform_per_second();
        let mut cargo = CargoMap::new();
        cargo.add(ResourceType::Coal, CargoAmount::new(0.025));
        cargo.add(ResourceType::Iron, CargoAmount::new(4.0));
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 0.5);
    }
}
