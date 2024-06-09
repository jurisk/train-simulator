use bevy::prelude::Resource;
use game_logic::server_state::ServerState;

pub mod channels;

#[derive(Resource)]
pub struct ServerStateResource(pub ServerState);

pub const DEFAULT_PORT: u16 = 5000;
