#![allow(clippy::needless_pass_by_value, clippy::module_name_repetitions)]

use bevy::app::{App, FixedUpdate};
use bevy::prelude::{
    EventReader, EventWriter, IntoSystemConfigs, Plugin, Res, ResMut, Resource, Time, Timer, info,
};
use bevy::time::TimerMode;
use shared_domain::client_command::{ClientCommand, NetworkCommand};
use shared_domain::server_response::{NetworkResponse, ServerResponse};
use uuid::Uuid;
use web_time::Duration;

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};

pub struct ClientPingPlugin {
    pub interval: Duration,
}

#[derive(Resource)]
struct PingTimer(Timer);

impl Plugin for ClientPingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PingTimer(Timer::new(self.interval, TimerMode::Repeating)))
            .add_systems(FixedUpdate, update_timer)
            .add_systems(FixedUpdate, handle_timer_just_finished.after(update_timer))
            .add_systems(FixedUpdate, handle_server_responses);
    }
}

fn update_timer(time: Res<Time>, timer: ResMut<PingTimer>) {
    let PingTimer(ref mut timer) = timer.into_inner();
    timer.tick(time.delta());
}

fn handle_server_responses(time: Res<Time>, mut server_messages: EventReader<ServerMessageEvent>) {
    let current_elapsed = time.elapsed();

    for message in server_messages.read() {
        let ServerMessageEvent { response } = message;
        if let ServerResponse::Network(NetworkResponse::Pong { id, elapsed }) = response {
            let delta = current_elapsed - *elapsed;
            info!("Ping {id}: {}ms", delta.as_millis());
        }
    }
}

fn handle_timer_just_finished(
    time: Res<Time>,
    timer: Res<PingTimer>,
    mut client_messages: EventWriter<ClientMessageEvent>,
) {
    let PingTimer(timer) = timer.into_inner();
    if timer.just_finished() {
        client_messages.send(ClientMessageEvent::new(ClientCommand::Network(
            NetworkCommand::Ping {
                id:      Uuid::new_v4(),
                elapsed: time.as_ref().elapsed(),
            },
        )));
    }
}
