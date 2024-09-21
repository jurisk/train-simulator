use bevy::log::info;
use bevy::prelude::{
    App, EventWriter, FixedUpdate, IntoSystemConfigs, Plugin, Res, ResMut, Resource, Time, Timer,
    TimerMode, in_state,
};
use game_ai::{ArtificialIntelligenceState, ai_commands};
use shared_domain::PlayerId;
use shared_domain::client_command::ClientCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::NoopMetrics;

use crate::communication::domain::ClientMessageEvent;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::states::ClientState;

#[derive(Resource)]
pub struct ArtificialIntelligenceTimer {
    timer: Option<Timer>,
}

impl ArtificialIntelligenceTimer {
    #[must_use]
    pub fn disabled() -> Self {
        Self { timer: None }
    }

    pub fn disable(&mut self) {
        info!("Disabling AI timer");
        self.timer = None;
    }

    pub fn enable(&mut self, seconds: f32) {
        info!("Enabling AI timer: {seconds} seconds");
        self.timer = Some(Timer::from_seconds(seconds, TimerMode::Repeating));
    }
}

#[derive(Resource, Default)]
pub struct ArtificialIntelligenceStateResource(ArtificialIntelligenceState);

pub struct ArtificialIntelligencePlugin;

impl Plugin for ArtificialIntelligencePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ArtificialIntelligenceTimer::disabled());
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
fn update_timer(time: Res<Time>, mut timer: ResMut<ArtificialIntelligenceTimer>) {
    if let Some(timer) = timer.timer.as_mut() {
        timer.tick(time.delta());
    }
}

#[expect(clippy::needless_pass_by_value)]
fn act_upon_timer(
    timer: Res<ArtificialIntelligenceTimer>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    player_id_resource: Res<PlayerIdResource>,
    game_state_resource: Res<GameStateResource>,
    mut ai_state_resource: ResMut<ArtificialIntelligenceStateResource>,
) {
    let ArtificialIntelligenceStateResource(ai_state) = &mut *ai_state_resource;
    if let Some(ref timer) = timer.timer {
        if timer.just_finished() {
            let PlayerIdResource(player_id) = *player_id_resource;
            let GameStateResource(game_state) = game_state_resource.as_ref();

            ai_step(player_id, game_state, &mut client_messages, ai_state);
        }
    }
}

fn ai_step(
    player_id: PlayerId,
    game_state: &GameState,
    client_messages: &mut EventWriter<ClientMessageEvent>,
    ai_state: &mut ArtificialIntelligenceState,
) {
    let commands = ai_commands(player_id, game_state, ai_state, &NoopMetrics::default());

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
