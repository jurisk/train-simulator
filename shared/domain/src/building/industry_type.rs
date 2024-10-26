#![allow(clippy::match_same_arms, clippy::enum_glob_use)]

use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};

use crate::building::WithRelativeTileCoverage;
use crate::building::building_info::WithCostToBuild;
use crate::building::industry_type::IndustryType::*;
use crate::building::resource_transform::{ResourceTransform, ResourceTransformItem};
use crate::cargo_map::CargoMap;
use crate::map_level::zoning::ZoningType;
use crate::map_level::zoning::ZoningType::{Industrial, Source};
use crate::resource_type::ResourceType;
use crate::resource_type::ResourceType::*;
use crate::tile_coords_xz::{TileCoordsXZ, TileDistance};
use crate::tile_coverage::TileCoverage;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash)]
pub enum IndustryType {
    CoalMine,
    OilWell,
    IronMine,
    NitrateMine,
    SulfurMine,
    Farm,
    Forestry,
    ClayPit,
    LimestoneMine,
    SandAndGravelQuarry,
    PowerPlant,
    CoalToOilPlant,
    SteelMill,
    ExplosivesPlant,
    FoodProcessingPlant,
    LumberMill,
    CellulosePlant,
    CementPlant,
    OilRefinery,
    ConcretePlant,
    TrainFactory,
    WeaponsFactory,
    AmmunitionFactory,
    MilitaryBase,
    ConstructionYard,
}

impl Debug for IndustryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CoalMine => write!(f, "CoalMine"),
            OilWell => write!(f, "OilWell"),
            IronMine => write!(f, "IronMine"),
            NitrateMine => write!(f, "NitrateMine"),
            SulfurMine => write!(f, "SulfurMine"),
            Farm => write!(f, "Farm"),
            Forestry => write!(f, "Forestry"),
            ClayPit => write!(f, "ClayPit"),
            LimestoneMine => write!(f, "LimestoneMine"),
            SandAndGravelQuarry => write!(f, "SandAndGravelQuarry"),
            PowerPlant => write!(f, "PowerPlant"),
            CoalToOilPlant => write!(f, "CoalToOilPlant"),
            SteelMill => write!(f, "SteelMill"),
            ExplosivesPlant => write!(f, "ExplosivesPlant"),
            FoodProcessingPlant => write!(f, "FoodProcessingPlant"),
            LumberMill => write!(f, "LumberMill"),
            CellulosePlant => write!(f, "CellulosePlant"),
            CementPlant => write!(f, "CementPlant"),
            OilRefinery => write!(f, "OilRefinery"),
            ConcretePlant => write!(f, "ConcretePlant"),
            TrainFactory => write!(f, "TrainFactory"),
            WeaponsFactory => write!(f, "WeaponsFactory"),
            AmmunitionFactory => write!(f, "AmmunitionFactory"),
            MilitaryBase => write!(f, "MilitaryBase"),
            ConstructionYard => write!(f, "ConstructionYard"),
        }
    }
}

const X1: f32 = 1.0;

impl IndustryType {
    #[must_use]
    pub const fn all() -> [Self; 24] {
        [
            CoalMine,
            OilWell,
            IronMine,
            NitrateMine,
            SulfurMine,
            Farm,
            Forestry,
            ClayPit,
            LimestoneMine,
            SandAndGravelQuarry,
            PowerPlant,
            CoalToOilPlant,
            SteelMill,
            ExplosivesPlant,
            FoodProcessingPlant,
            LumberMill,
            CementPlant,
            OilRefinery,
            ConcretePlant,
            TrainFactory,
            WeaponsFactory,
            AmmunitionFactory,
            MilitaryBase,
            ConstructionYard,
        ]
    }

    #[must_use]
    pub fn required_zoning(self) -> Option<ZoningType> {
        match self {
            CoalMine => Some(Source(Coal)),
            OilWell => Some(Source(Oil)),
            IronMine => Some(Source(Iron)),
            NitrateMine => Some(Source(Nitrates)),
            SulfurMine => Some(Source(Sulfur)),
            Farm => Some(Source(FarmProducts)),
            Forestry => Some(Source(Wood)),
            ClayPit => Some(Source(Clay)),
            LimestoneMine => Some(Source(Limestone)),
            SandAndGravelQuarry => Some(Source(SandAndGravel)),
            PowerPlant => Some(Industrial),
            CoalToOilPlant => Some(Industrial),
            SteelMill => Some(Industrial),
            ExplosivesPlant => Some(Industrial),
            FoodProcessingPlant => Some(Industrial),
            LumberMill => Some(Industrial),
            CellulosePlant => Some(Industrial),
            CementPlant => Some(Industrial),
            OilRefinery => Some(Industrial),
            ConcretePlant => Some(Industrial),
            TrainFactory => Some(Industrial),
            WeaponsFactory => Some(Industrial),
            AmmunitionFactory => Some(Industrial),
            MilitaryBase => Some(Industrial),
            ConstructionYard => Some(Industrial),
        }
    }

    #[must_use]
    fn something_resource_types<F>(self, f: F) -> Vec<ResourceType>
    where
        F: FnOnce(ResourceTransform) -> Vec<ResourceTransformItem>,
    {
        f(self.transform_per_second())
            .iter()
            .map(|item| item.resource)
            .collect()
    }

    #[must_use]
    pub fn produces(self, resource: ResourceType) -> bool {
        self.output_resource_types().contains(&resource)
    }

    #[must_use]
    pub fn consumes(self, resource: ResourceType) -> bool {
        self.input_resource_types().contains(&resource)
    }

    #[must_use]
    pub fn input_resource_types(self) -> Vec<ResourceType> {
        self.something_resource_types(|transform| transform.inputs)
    }

    #[must_use]
    pub fn output_resource_types(self) -> Vec<ResourceType> {
        self.something_resource_types(|transform| transform.outputs)
    }

    #[must_use]
    pub fn supply_range_in_tiles(self) -> Option<TileDistance> {
        match self {
            ConstructionYard => Some(128),
            MilitaryBase => Some(32),
            _ => None,
        }
    }

    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn transform_per_second(self) -> ResourceTransform {
        match self {
            SteelMill => {
                // https://marketrealist.com/2015/01/coke-fit-steelmaking-process/
                // Later: In theory, it is 2 iron to 1 coal, but we simplified it for now to avoid so much legwork for the player.
                ResourceTransform::make(vec![(Iron, X1), (Coal, X1)], vec![(Steel, X1)])
            },
            PowerPlant => ResourceTransform::make(vec![(Coal, X1)], vec![]),
            CoalToOilPlant => ResourceTransform::make(vec![(Coal, X1)], vec![(Oil, X1)]),
            ExplosivesPlant => {
                ResourceTransform::make(vec![(Nitrates, X1), (Sulfur, X1), (Cellulose, X1)], vec![
                    (Explosives, X1),
                ])
            },
            FoodProcessingPlant => {
                ResourceTransform::make(vec![(FarmProducts, X1)], vec![(Food, X1)])
            },
            LumberMill => ResourceTransform::make(vec![(Wood, X1)], vec![(Timber, X1)]),
            CellulosePlant => ResourceTransform::make(vec![(Wood, X1)], vec![(Cellulose, X1)]),
            CementPlant => {
                ResourceTransform::make(vec![(Clay, X1), (Limestone, X1)], vec![(Cement, X1)])
            },
            OilRefinery => ResourceTransform::make(vec![(Oil, X1)], vec![(Fuel, X1)]),
            ConcretePlant => {
                ResourceTransform::make(vec![(Cement, X1), (SandAndGravel, X1)], vec![(
                    Concrete, X1,
                )])
            },
            TrainFactory => ResourceTransform::make(vec![(Steel, X1)], vec![]),
            WeaponsFactory => {
                ResourceTransform::make(vec![(Steel, X1)], vec![(ArtilleryWeapons, X1)])
            },
            AmmunitionFactory => {
                ResourceTransform::make(vec![(Steel, X1), (Explosives, X1)], vec![(Ammunition, X1)])
            },
            MilitaryBase => {
                ResourceTransform::make_warehouse(&[Ammunition, Food, Fuel, ArtilleryWeapons])
            },
            ConstructionYard => ResourceTransform::make_warehouse(&[Concrete, Steel, Timber]),
            IronMine | CoalMine | OilWell | NitrateMine | SulfurMine | Farm | Forestry
            | ClayPit | LimestoneMine | SandAndGravelQuarry => {
                match self.required_zoning() {
                    Some(Source(resource)) => ResourceTransform::make(vec![], vec![(resource, X1)]),
                    other => {
                        panic!("Unexpected zoning for {self:?}: {other:?}")
                    },
                }
            },
        }
    }
}

impl WithRelativeTileCoverage for IndustryType {
    #[must_use]
    fn relative_tiles_used(&self) -> TileCoverage {
        TileCoverage::rectangular_odd(TileCoordsXZ::ZERO, 3, 3)
    }
}

impl WithCostToBuild for IndustryType {
    #[must_use]
    fn cost_to_build(&self) -> (IndustryType, CargoMap) {
        (
            ConstructionYard,
            CargoMap::from([(Concrete, 8.0), (Steel, 4.0)]),
        )
    }
}

#[cfg(test)]
#[expect(clippy::float_cmp)]
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
        assert_eq!(utilisation, 1f32);
    }

    #[test]
    fn test_iron_works_empty() {
        let transform = IndustryType::SteelMill.transform_per_second();
        let cargo = CargoMap::new();
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 0f32);
    }

    #[test]
    fn test_iron_works_one_component_only_other_empty() {
        let transform = IndustryType::SteelMill.transform_per_second();
        let mut cargo = CargoMap::new();
        cargo.add(ResourceType::Coal, CargoAmount::new(4.0));
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 0f32);
    }

    #[test]
    fn test_iron_abundance() {
        let transform = IndustryType::SteelMill.transform_per_second();
        let mut cargo = CargoMap::new();
        cargo.add(ResourceType::Coal, CargoAmount::new(4.0));
        cargo.add(ResourceType::Iron, CargoAmount::new(4.0));
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 1f32);
    }

    #[test]
    fn test_iron_partial() {
        let transform = IndustryType::SteelMill.transform_per_second();
        let mut cargo = CargoMap::new();
        cargo.add(ResourceType::Coal, CargoAmount::new(0.025));
        cargo.add(ResourceType::Iron, CargoAmount::new(4.0));
        let utilisation = transform.calculate_utilisation_percentage(&cargo, 0.5);
        assert_eq!(utilisation, 0.25f32);
    }
}
