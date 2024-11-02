mod industries;
mod military;
mod resource_links;
mod stations;
mod supply_chains;
mod transports;

use log::{error, trace};
use shared_domain::PlayerId;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;

use crate::ArtificialIntelligenceState;
use crate::oct2025::military::MilitaryBasesAI;
use crate::oct2025::supply_chains::BuildSupplyChains;

enum GoalResult {
    SendCommands(Vec<GameCommand>),
    RepeatInvocation,
    TryAgainLater,
    Finished,
    Error(String),
}

fn invoke_to_finished<F>(mut f: F) -> GoalResult
where
    F: FnMut() -> GoalResult,
{
    loop {
        let result = f();
        match result {
            GoalResult::RepeatInvocation => continue,
            other => return other,
        }
    }
}

trait Goal {
    fn commands(
        &mut self,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> GoalResult;
}

impl ArtificialIntelligenceState for Oct2025ArtificialIntelligenceState {
    fn ai_commands(
        &mut self,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>> {
        for goal in &mut self.pending_goals {
            let result = invoke_to_finished(|| goal.commands(self.player_id, game_state, metrics));
            match result {
                GoalResult::SendCommands(commands) => {
                    return Some(commands);
                },
                GoalResult::RepeatInvocation => {
                    error!("Unexpected result from `invoke_to_finished`");
                },
                GoalResult::TryAgainLater => {
                    // This goal did not have enough resources, let us not do anything and keep with it.
                    return None;
                },
                GoalResult::Finished => {
                    // We move to the next goal
                },
                GoalResult::Error(error) => {
                    error!("Error in AI goal: {error}");
                },
            }
        }

        trace!("AI has nothing to do");
        None
    }
}

#[expect(clippy::module_name_repetitions)]
pub struct Oct2025ArtificialIntelligenceState {
    player_id:     PlayerId,
    pending_goals: Vec<Box<dyn Goal + Send + Sync>>,
}

impl Oct2025ArtificialIntelligenceState {
    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn new(player_id: PlayerId, game_state: &GameState) -> Self {
        let construction_yards = game_state
            .building_state()
            .find_industry_building_by_owner_and_type(player_id, IndustryType::ConstructionYard)
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(
            construction_yards.len(),
            1,
            "Expected exactly one construction yard for player {player_id}"
        );
        let construction_yard = construction_yards[0];
        let construction_yard_location = construction_yard.reference_tile();
        let construction_yard_id = construction_yard.id();
        let pending_goals: Vec<Box<dyn Goal + Send + Sync>> = vec![
            Box::new(BuildSupplyChains::for_known_target(
                game_state.supply_chain(),
                IndustryType::ConstructionYard,
                construction_yard_location,
                construction_yard_id,
            )) as Box<dyn Goal + Send + Sync>,
            Box::new(MilitaryBasesAI::new()) as Box<dyn Goal + Send + Sync>,
        ];

        Self {
            player_id,
            pending_goals,
        }
    }
}
