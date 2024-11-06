use std::time::Duration;

use bevy::log::{debug, info};
use bevy::prelude::{
    App, EventReader, EventWriter, FixedUpdate, IntoSystemConfigs, Plugin, Res, ResMut, Resource,
    Time, Timer, TimerMode, in_state,
};
use bevy::utils::HashMap;
use game_ai::ArtificialIntelligenceState;
use game_ai::oct2025::Oct2025ArtificialIntelligenceState;
use shared_domain::PlayerId;
use shared_domain::client_command::ClientCommand;
use shared_domain::game_state::GameState;
use shared_domain::metrics::NoopMetrics;
use shared_domain::server_response::ServerResponse;

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::GameStateResource;
use crate::states::ClientState;

#[derive(Resource, Default)]
pub struct ArtificialIntelligenceResource {
    map: HashMap<PlayerId, (Timer, Box<dyn ArtificialIntelligenceState>)>,
}

impl ArtificialIntelligenceResource {
    pub fn disable(&mut self, player_id: PlayerId) {
        info!("Disabling AI timer for player {player_id}");
        if let Some((timer, _)) = self.map.get_mut(&player_id) {
            timer.set_duration(Duration::MAX);
        }
    }

    pub fn enable(&mut self, player_id: PlayerId, seconds: f32, game_state: &GameState) {
        // Insert a new AI state if it doesn't exist
        info!("Enabling AI timer for player {player_id}: {seconds} seconds");
        let duration = Duration::from_secs_f32(seconds);
        if let Some((timer, _)) = self.map.get_mut(&player_id) {
            timer.set_duration(duration);
        } else {
            let timer = Timer::new(duration, TimerMode::Repeating);
            // TODO: Make switchable between different AI implementations when you have more than one
            let ai_state = Oct2025ArtificialIntelligenceState::new(player_id, game_state);
            let state = Box::new(ai_state);
            self.map.insert(player_id, (timer, state));
        }
    }
}

pub struct ArtificialIntelligencePlugin;

impl Plugin for ArtificialIntelligencePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ArtificialIntelligenceResource::default());
        app.add_systems(
            FixedUpdate,
            update_timer.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            act_upon_timer.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            send_server_message_events_to_ais.run_if(in_state(ClientState::Playing)),
        );
    }
}

fn send_server_message_events_to_ais(
    mut artificial_intelligence_resource: ResMut<ArtificialIntelligenceResource>,
    mut server_message_events: EventReader<ServerMessageEvent>,
) {
    for event in server_message_events.read() {
        // Later: Not all server message events may be aimed at all AIs - right now AIs may be getting server message events that are off-topic for them. Perhaps the messages should have a marker which player they are for. But this adds complexity.
        let ServerMessageEvent { response } = event;

        if let ServerResponse::Game(_game_id, game_message) = response {
            for (_player_id, (_, ai_state)) in &mut artificial_intelligence_resource.map {
                ai_state.notify_of_response(game_message);
            }
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
fn update_timer(time: Res<Time>, mut timers: ResMut<ArtificialIntelligenceResource>) {
    for (timer, _) in timers.map.values_mut() {
        timer.tick(time.delta());
    }
}

#[expect(clippy::needless_pass_by_value)]
fn act_upon_timer(
    mut artificial_intelligence_resource: ResMut<ArtificialIntelligenceResource>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    game_state_resource: Res<GameStateResource>,
) {
    for (_player_id, (timer, ref mut state)) in &mut artificial_intelligence_resource.map {
        if timer.just_finished() {
            let GameStateResource(game_state) = game_state_resource.as_ref();
            ai_step(game_state, &mut client_messages, state.as_mut());
        }
    }
}

fn ai_step(
    game_state: &GameState,
    client_messages: &mut EventWriter<ClientMessageEvent>,
    ai_state: &mut dyn ArtificialIntelligenceState,
) {
    let commands = ai_state.ai_commands(game_state, &NoopMetrics::default());

    if let Some(commands) = commands {
        for command in commands {
            info!("AI chose command: {:?}", command);
            client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                game_state.game_id(),
                command,
            )));
        }
    } else {
        debug!("AI has nothing to do");
    }
}
