use bevy::app::App;
use bevy::prelude::Plugin;

use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};

pub mod domain;

#[allow(clippy::module_name_repetitions)]
pub struct CommunicationPlugin;

impl Plugin for CommunicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ClientMessageEvent>();
        app.add_event::<ServerMessageEvent>();
    }
}
