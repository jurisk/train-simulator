use log::info;
use shared_domain::game_state::GameState;
use shared_domain::map_level::MapLevel;
use shared_protocol::game_selection::{ClientMessage, ServerMessage};

#[allow(clippy::module_name_repetitions, clippy::missing_panics_doc)]
#[must_use]
pub fn server_logic(client_message: &ClientMessage) -> Vec<ServerMessage> {
    match client_message {
        ClientMessage::JoinGame => {
            let level_json = include_str!("../assets/map_levels/default.json");
            let level = serde_json::from_str::<MapLevel>(level_json)
                .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));

            assert!(level.is_valid());

            let game_state = GameState { map_level: level };

            info!("Simulating server responding to JoinGame with GameJoined");

            vec![ServerMessage::GameJoined { game_state }]
        },
    }
}
