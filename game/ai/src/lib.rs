pub mod sep2025;

use std::collections::BTreeSet;

use shared_domain::PlayerId;
use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::Metrics;

pub(crate) type SetOfTwo<T> = BTreeSet<T>;

pub trait ArtificialIntelligenceState: Send + Sync {
    fn ai_commands(
        &mut self,
        player_id: PlayerId,
        game_state: &GameState,
        metrics: &dyn Metrics,
    ) -> Option<Vec<GameCommand>>;
}
