pub mod oct2025;

use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;

pub trait ArtificialIntelligenceState: Send + Sync {
    fn ai_commands(
        &mut self,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>>;
}
