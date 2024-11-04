use std::collections::HashMap;

use shared_domain::building::industry_type::IndustryType;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::resource_type::ResourceType;
use shared_domain::server_response::GameResponse;
use shared_domain::supply_chain::SupplyChain;
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
            GoalResult::Error(format!(
                "Missing industry state for resource link: {from_industry:?} -> {to_industry:?}"
            ))
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

        GoalResult::Finished
    }

    fn notify_of_response(&mut self, response: &GameResponse) {
        for state in self.industry_states.values_mut() {
            state.notify_of_response(response);
        }

        for state in self.resource_link_states.values_mut() {
            state.notify_of_response(response);
        }
    }
}

impl BuildSupplyChain {
    #[must_use]
    pub fn with_built_target(
        supply_chain: &SupplyChain,
        resource_type: ResourceType,
        target_type: IndustryType,
        target_location: TileCoordsXZ,
        target_id: IndustryBuildingId,
    ) -> Self {
        let industries =
            supply_chain.industries_for_resource_and_target(resource_type, target_type);

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
    // Later: Think carefully if race conditions are still possible and whether we should thus first ensure the target industry & its station are built first, as that is shared between all the sub-goals!
    sub_goals: Vec<BuildSupplyChain>,
}

impl BuildSupplyChains {
    #[must_use]
    pub(crate) fn for_known_target(
        supply_chain: &SupplyChain,
        target_type: IndustryType,
        target_location: TileCoordsXZ,
        target_id: IndustryBuildingId,
    ) -> Self {
        let resources = supply_chain.input_resource_types(target_type);

        let sub_goals = resources
            .into_iter()
            .map(|resource| {
                BuildSupplyChain::with_built_target(
                    supply_chain,
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
            let result = invoke_to_finished(|| sub_goal.commands(player_id, game_state, metrics));
            match result {
                GoalResult::Finished => {},
                other => return other,
            }
        }

        GoalResult::Finished
    }

    fn notify_of_response(&mut self, response: &GameResponse) {
        for sub_goal in &mut self.sub_goals {
            sub_goal.notify_of_response(response);
        }
    }
}
