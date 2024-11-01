use std::collections::HashMap;

use log::error;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::resource_type::ResourceType;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::{IndustryBuildingId, PlayerId};

use crate::oct2025::industries::IndustryState;
use crate::oct2025::resource_links::{ResourceLinkState, resource_links};
use crate::oct2025::{Goal, GoalResult, invoke_to_finished};

#[derive(Clone)]
struct BuildSupplyChain {
    target_location:      TileCoordsXZ,
    industry_states:      HashMap<IndustryType, IndustryState>,
    resource_link_states: HashMap<(IndustryType, ResourceType, IndustryType), ResourceLinkState>,
}

impl BuildSupplyChain {
    #[expect(clippy::too_many_arguments)]
    fn resource_link_commands(
        industry_states: &HashMap<IndustryType, IndustryState>,
        state: &mut ResourceLinkState,
        from_industry: IndustryType,
        resource: ResourceType,
        to_industry: IndustryType,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> GoalResult {
        let from_industry_state = industry_states.get(&from_industry);
        let to_industry_state = industry_states.get(&to_industry);
        if let (Some(from_industry_state), Some(to_industry_state)) =
            (from_industry_state, to_industry_state)
        {
            state.commands(
                from_industry_state,
                resource,
                to_industry_state,
                player_id,
                game_state,
                metrics,
            )
        } else {
            error!(
                "Missing industry state for resource link: {from_industry:?} -> {to_industry:?}"
            );
            GoalResult::Done
        }
    }
}

impl Goal for BuildSupplyChain {
    fn commands(
        &mut self,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> GoalResult {
        for (industry, state) in &mut self.industry_states {
            if let GoalResult::SendCommands(responses) = invoke_to_finished(|| {
                state.commands(*industry, player_id, game_state, self.target_location)
            }) {
                return GoalResult::SendCommands(responses);
            }
        }

        for ((from_industry, resource, to_industry), state) in &mut self.resource_link_states {
            if let GoalResult::SendCommands(responses) = invoke_to_finished(|| {
                Self::resource_link_commands(
                    &self.industry_states,
                    state,
                    *from_industry,
                    *resource,
                    *to_industry,
                    player_id,
                    game_state,
                    metrics,
                )
            }) {
                return GoalResult::SendCommands(responses);
            }
        }

        GoalResult::Done
    }
}

// TODO: You can generate this from the industry definitions - this is surely needed if you want to have moddable supply chains
fn industries_for_resource_and_target(
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
            panic!(
                "Unsupported resource and target combination: {resource_type:?} -> {target_type:?}"
            )
        },
    }
}

impl BuildSupplyChain {
    #[must_use]
    pub fn with_built_target(
        resource_type: ResourceType,
        target_type: IndustryType,
        target_location: TileCoordsXZ,
        target_id: IndustryBuildingId,
    ) -> Self {
        let industries = industries_for_resource_and_target(resource_type, target_type);

        let mut industry_states: HashMap<IndustryType, IndustryState> = industries
            .iter()
            .map(|industry| (*industry, IndustryState::NothingDone))
            .collect();

        industry_states.insert(
            target_type,
            IndustryState::IndustryBuilt(target_id, target_location),
        );

        let resource_link_states = resource_links(&industries)
            .into_iter()
            .map(|(from_industry, resource, to_industry)| {
                (
                    (from_industry, resource, to_industry),
                    ResourceLinkState::Pending,
                )
            })
            .collect();

        Self {
            target_location,
            industry_states,
            resource_link_states,
        }
    }
}

#[derive(Clone)]
pub(crate) struct BuildSupplyChains {
    sub_goals: Vec<BuildSupplyChain>,
}

impl BuildSupplyChains {
    #[must_use]
    pub(crate) fn for_known_target(
        target_type: IndustryType,
        target_location: TileCoordsXZ,
        target_id: IndustryBuildingId,
    ) -> Self {
        let resources = match target_type {
            IndustryType::ConstructionYard | IndustryType::MilitaryBase => {
                target_type.input_resource_types()
            },
            _ => panic!("Unsupported target type: {target_type:?}"),
        };

        let sub_goals = resources
            .into_iter()
            .map(|resource| {
                BuildSupplyChain::with_built_target(
                    resource,
                    target_type,
                    target_location,
                    target_id,
                )
            })
            .collect();

        Self { sub_goals }
    }
}

impl Goal for BuildSupplyChains {
    fn commands(
        &mut self,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> GoalResult {
        for sub_goal in &mut self.sub_goals {
            if let GoalResult::SendCommands(commands) =
                invoke_to_finished(|| sub_goal.commands(player_id, game_state, metrics))
            {
                return GoalResult::SendCommands(commands);
            }
        }

        GoalResult::Done
    }
}
