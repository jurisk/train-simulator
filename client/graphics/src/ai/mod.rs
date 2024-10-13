use bevy::log::info;
use bevy::prelude::{
    App, EventWriter, FixedUpdate, IntoSystemConfigs, Plugin, Res, ResMut, Resource, Time, Timer,
    TimerMode, in_state,
};
use bevy::utils::HashMap;
use game_ai::ArtificialIntelligenceState;
use shared_domain::PlayerId;
use shared_domain::client_command::ClientCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::NoopMetrics;

use crate::communication::domain::ClientMessageEvent;
use crate::game::GameStateResource;
use crate::states::ClientState;

#[derive(Resource)]
pub struct ArtificialIntelligenceTimers {
    timers: HashMap<PlayerId, Timer>,
}

impl ArtificialIntelligenceTimers {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            timers: HashMap::new(),
        }
    }

    pub fn disable(&mut self, player_id: PlayerId) {
        info!("Disabling AI timer for player {player_id}");
        self.timers.remove(&player_id);
    }

    pub fn enable(&mut self, player_id: PlayerId, seconds: f32) {
        info!("Enabling AI timer for player {player_id}: {seconds} seconds");
        self.timers.insert(
            player_id,
            Timer::from_seconds(seconds, TimerMode::Repeating),
        );
    }
}

#[derive(Resource, Default)]
pub struct ArtificialIntelligenceStateResource(ArtificialIntelligenceState);

pub struct ArtificialIntelligencePlugin;

impl Plugin for ArtificialIntelligencePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ArtificialIntelligenceTimers::empty());
        app.add_systems(
            FixedUpdate,
            update_timer.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            act_upon_timer.run_if(in_state(ClientState::Playing)),
        );
        app.insert_resource(ArtificialIntelligenceStateResource::default());
    }
}

#[expect(clippy::needless_pass_by_value)]
fn update_timer(time: Res<Time>, mut timers: ResMut<ArtificialIntelligenceTimers>) {
    for timer in timers.timers.values_mut() {
        timer.tick(time.delta());
    }
}

#[expect(clippy::needless_pass_by_value)]
fn act_upon_timer(
    timers: Res<ArtificialIntelligenceTimers>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    game_state_resource: Res<GameStateResource>,
    mut ai_state_resource: ResMut<ArtificialIntelligenceStateResource>,
) {
    let ArtificialIntelligenceStateResource(ai_state) = &mut *ai_state_resource;
    for (player_id, timer) in &timers.timers {
        if timer.just_finished() {
            let GameStateResource(game_state) = game_state_resource.as_ref();
            ai_step(*player_id, game_state, &mut client_messages, ai_state);
        }
    }
}

fn ai_step(
    player_id: PlayerId,
    game_state: &GameState,
    client_messages: &mut EventWriter<ClientMessageEvent>,
    ai_state: &mut ArtificialIntelligenceState,
) {
    let commands = ai_state.ai_commands(player_id, game_state, &NoopMetrics::default());

    if let Some(commands) = commands {
        for command in commands {
            info!("AI chose command: {:?}", command);
            client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                game_state.game_id(),
                command,
            )));
        }
    } else {
        info!("AI has nothing to do");
    }
}
