pub mod oct2025;

use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;

// TODO: Move the `metrics` part to actually already be in the state so we don't have to pass it in every time. This was not trivial though when we tried it before.
pub trait ArtificialIntelligenceState: Send + Sync {
    fn ai_commands(
        &mut self,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>>;
}
