#![expect(clippy::module_name_repetitions)]
use shared_domain::PlayerId;
use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;

use crate::ArtificialIntelligenceState;

pub struct Oct2025ArtificialIntelligenceState {}

impl Oct2025ArtificialIntelligenceState {
    #[must_use]
    pub fn new(_game_state: &GameState) -> Self {
        Self {}
    }
}

impl ArtificialIntelligenceState for Oct2025ArtificialIntelligenceState {
    fn ai_commands(
        &mut self,
        _player_id: PlayerId,
        _game_state: &GameState,
        _metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>> {
        // TODO HIGH: have some state machine where first obtain steel, then timber, then concrete, then etc
        None
    }
}
