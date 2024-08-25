use bevy::app::{App, FixedUpdate};
use bevy::prelude::{
    in_state, info, IntoSystemConfigs, Plugin, Res, ResMut, Resource, Time, Timer, TimerMode,
};

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
    }
}

#[allow(clippy::needless_pass_by_value)]
fn update_timer(time: Res<Time>, mut timer: ResMut<ArtificialIntelligenceTimer>) {
    if let Some(timer) = timer.timer.as_mut() {
        timer.tick(time.delta());
    }
}

#[allow(clippy::needless_pass_by_value)]
fn act_upon_timer(timer: Res<ArtificialIntelligenceTimer>) {
    if let Some(ref timer) = timer.timer {
        if timer.just_finished() {
            info!("AI timer just finished");
        }
    }
}
