use bevy::prelude::Resource;
use game_logic::server_state::ServerState;

pub mod channels;

#[derive(Resource)]
pub struct ServerStateResource(pub ServerState);
