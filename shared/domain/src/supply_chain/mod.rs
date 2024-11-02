use log::error;

use crate::building::industry_type::IndustryType;
use crate::resource_type::ResourceType;

// TODO HIGH: Support custom supply chains - one WW1 one, and one peaceful "just build industry" one
#[derive(Debug, PartialEq, Clone)]
pub struct SupplyChain {}

impl SupplyChain {
    #[must_use]
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    #[must_use]
    pub fn industries_for_industry(&self, target_type: IndustryType) -> Vec<IndustryType> {
        let mut results = vec![];
        for resource in self.input_resource_types(target_type) {
            results.extend(self.industries_for_resource_and_target(resource, target_type));
        }
        results
    }

    // TODO: You can generate this from the industry definitions - this is surely needed if you want to have moddable supply chains
    #[must_use]
    pub fn industries_for_resource_and_target(
        &self,
        resource_type: ResourceType,
        target_type: IndustryType,
    ) -> Vec<IndustryType> {
        match (resource_type, target_type) {
            (ResourceType::Steel, IndustryType::ConstructionYard) => {
                vec![
                    IndustryType::IronMine,
                    IndustryType::CoalMine,
                    IndustryType::SteelMill,
                    IndustryType::ConstructionYard,
                ]
            },
            (ResourceType::Timber, IndustryType::ConstructionYard) => {
                vec![
                    IndustryType::Forestry,
                    IndustryType::LumberMill,
                    IndustryType::ConstructionYard,
                ]
            },
            (ResourceType::Concrete, IndustryType::ConstructionYard) => {
                vec![
                    IndustryType::ClayPit,
                    IndustryType::SandAndGravelQuarry,
                    IndustryType::LimestoneMine,
                    IndustryType::CementPlant,
                    IndustryType::ConcretePlant,
                    IndustryType::ConstructionYard,
                ]
            },
            (ResourceType::ArtilleryWeapons, IndustryType::MilitaryBase) => {
                vec![
                    IndustryType::CoalMine,
                    IndustryType::IronMine,
                    IndustryType::SteelMill,
                    IndustryType::WeaponsFactory,
                    IndustryType::MilitaryBase,
                ]
            },
            (ResourceType::Food, IndustryType::MilitaryBase) => {
                vec![
                    IndustryType::Farm,
                    IndustryType::FoodProcessingPlant,
                    IndustryType::MilitaryBase,
                ]
            },
            (ResourceType::Ammunition, IndustryType::MilitaryBase) => {
                vec![
                    IndustryType::Forestry,
                    IndustryType::CellulosePlant,
                    IndustryType::AmmunitionFactory,
                    IndustryType::ExplosivesPlant,
                    IndustryType::NitrateMine,
                    IndustryType::SulfurMine,
                    IndustryType::IronMine,
                    IndustryType::CoalMine,
                    IndustryType::SteelMill,
                    IndustryType::MilitaryBase,
                ]
            },
            (ResourceType::Fuel, IndustryType::MilitaryBase) => {
                vec![
                    IndustryType::OilWell,
                    IndustryType::OilRefinery,
                    IndustryType::MilitaryBase,
                ]
            },
            _ => {
                error!(
                    "Unsupported resource and target combination: {resource_type:?} -> {target_type:?}"
                );
                vec![]
            },
        }
    }

    #[must_use]
    pub fn input_resource_types(&self, target_type: IndustryType) -> Vec<ResourceType> {
        target_type.input_resource_types()
    }
}
