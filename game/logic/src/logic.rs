use log::info;
use shared_domain::game_state::GameState;
use shared_domain::map_level::MapLevel;
use shared_protocol::client_command::{ClientCommand, LobbyCommand};
use shared_protocol::server_response::{GameInfo, GameResponse, LobbyResponse, ServerResponse};

// TODO: Should be more stateful and separated into Authentication, Lobby, Game
#[allow(clippy::module_name_repetitions, clippy::missing_panics_doc)]
#[must_use]
pub fn server_logic(client_command: &ClientCommand) -> Vec<ServerResponse> {
    match client_command {
        ClientCommand::Lobby(lobby_command) => {
            match lobby_command {
                LobbyCommand::JoinGame(game_id) => {
                    let level_json = include_str!("../assets/map_levels/default.json");
                    let level = serde_json::from_str::<MapLevel>(level_json)
                        .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));

                    assert!(level.is_valid());

                    let game_state = GameState { map_level: level };

                    info!("Simulating server responding to JoinGame with GameJoined");

                    vec![
                        ServerResponse::Lobby(LobbyResponse::GameJoined(GameInfo {
                            game_id: *game_id,
                        })),
                        ServerResponse::Game(GameResponse::State(game_state)),
                    ]
                },
                _ => todo!(), // TODO: Implement other handling
            }
        },
        _ => todo!(), // TODO: Implement other handling
    }
}
