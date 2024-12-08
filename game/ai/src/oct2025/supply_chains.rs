use std::collections::HashMap;

use log::trace;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::resource_type::ResourceType;
use shared_domain::server_response::GameResponse;
use shared_domain::supply_chain::SupplyChain;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::{IndustryBuildingId, PlayerId, StationId};

use crate::oct2025::industries::{BuildIndustry, BuildIndustryState};
use crate::oct2025::resource_links::{BuildResourceLink, ResourceLinkState, resource_links};
use crate::oct2025::{Goal, GoalResult, invoke_to_finished};

#[derive(Clone, Debug)]
struct BuildSupplyChain {
    industry_states:      HashMap<IndustryType, BuildIndustry>,
    resource_link_states:
        HashMap<(IndustryType, ResourceType, IndustryType), Option<BuildResourceLink>>,
}

fn lookup_station_id(industry_state: &BuildIndustry) -> Option<StationId> {
    if let BuildIndustryState::StationBuilt(_industry_building_id, _location, station_id) =
        industry_state.state
    {
        Some(station_id)
    } else {
        trace!("No station built for {industry_state:?}");
        None
    }
}

impl Goal for BuildSupplyChain {
    fn commands(
        &mut self,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> GoalResult {
        for state in &mut self.industry_states.values_mut() {
            if let GoalResult::SendCommands(responses) =
                invoke_to_finished(|| state.commands(player_id, game_state, metrics))
            {
                return GoalResult::SendCommands(responses);
            }
        }

        for ((from_industry, resource, to_industry), resource_link) in
            &mut self.resource_link_states
        {
            if resource_link.is_none() {
                if let (Some(from_industry), Some(to_industry)) = (
                    self.industry_states.get(from_industry),
                    self.industry_states.get(to_industry),
                ) {
                    if let (Some(from_station_id), Some(to_station_id)) = (
                        lookup_station_id(from_industry),
                        lookup_station_id(to_industry),
                    ) {
                        *resource_link = Some(BuildResourceLink {
                            from_station_id,
                            resource: *resource,
                            to_station_id,
                            state: ResourceLinkState::Pending,
                        });
                    }
                }
            }
        }

        for resource_link in &mut self.resource_link_states.values_mut().flatten() {
            if let GoalResult::SendCommands(responses) =
                invoke_to_finished(|| resource_link.commands(player_id, game_state, metrics))
            {
                return GoalResult::SendCommands(responses);
            }
        }

        GoalResult::Finished
    }

    fn notify_of_response(&mut self, response: &GameResponse) {
        for industry_state in self.industry_states.values_mut() {
            industry_state.notify_of_response(response);
        }

        for resource_link in self.resource_link_states.values_mut().flatten() {
            resource_link.notify_of_response(response);
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

        let mut industry_states: HashMap<IndustryType, BuildIndustry> = industries
            .iter()
            .map(|&industry_type| {
                (industry_type, BuildIndustry {
                    industry_type,
                    target_location,
                    state: BuildIndustryState::NothingDone,
                })
            })
            .collect();

        industry_states.insert(target_type, BuildIndustry {
            industry_type: target_type,
            target_location,
            state: BuildIndustryState::IndustryBuilt(target_id, target_location),
        });

        let resource_link_states = resource_links(&industries)
            .into_iter()
            .map(|(from_industry, resource, to_industry)| {
                ((from_industry, resource, to_industry), None)
            })
            .collect();

        Self {
            industry_states,
            resource_link_states,
        }
    }
}

#[derive(Clone, Debug)]
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
