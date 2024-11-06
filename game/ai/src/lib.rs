pub mod oct2025;

use std::fmt::Debug;

use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;
use shared_domain::server_response::GameResponse;

// TODO: Move the `metrics` part to actually already be in the state so we don't have to pass it in every time. This was not trivial though when we tried it before.
pub trait ArtificialIntelligenceState: Send + Sync + Debug {
    fn ai_commands(
        &mut self,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>>;

    fn notify_of_response(&mut self, response: &GameResponse);
}
