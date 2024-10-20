#![expect(clippy::module_name_repetitions)]

use shared_domain::building::industry_type::IndustryType;
use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::{IndustryBuildingId, PlayerId};

use crate::ArtificialIntelligenceState;

#[derive(Copy, Clone)]
enum Goal {
    SteelToConstructionYard(IndustryBuildingId),
    TimberToConstructionYard(IndustryBuildingId),
    ConcreteToConstructionYard(IndustryBuildingId),
}

pub struct Oct2025ArtificialIntelligenceState {
    player_id:     PlayerId,
    pending_goals: Vec<Goal>,
}

impl Oct2025ArtificialIntelligenceState {
    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn new(player_id: PlayerId, game_state: &GameState) -> Self {
        let construction_yards = game_state
            .building_state()
            .find_industry_building_by_owner_and_type(player_id, IndustryType::ConstructionYard);
        assert_eq!(
            construction_yards.len(),
            1,
            "Expected exactly one construction yard for player {player_id}"
        );
        let construction_yard = construction_yards[0];
        let construction_yard_id = construction_yard.id();
        Self {
            player_id,
            pending_goals: vec![
                Goal::SteelToConstructionYard(construction_yard_id),
                Goal::TimberToConstructionYard(construction_yard_id),
                Goal::ConcreteToConstructionYard(construction_yard_id),
            ],
        }
    }
}

impl Oct2025ArtificialIntelligenceState {
    fn commands_for_goal(&mut self, goal: Goal) -> Vec<GameCommand> {
        vec![]
    }
}

impl ArtificialIntelligenceState for Oct2025ArtificialIntelligenceState {
    fn ai_commands(
        &mut self,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>> {
        let next_goal = self.pending_goals.first().copied();
        match next_goal {
            None => None,
            Some(goal) => {
                // TODO: This assumes that the goal is always achieved, that all commands succeed. That's wrong.
                self.pending_goals.remove(0);
                Some(self.commands_for_goal(goal))
            },
        }
    }
}
