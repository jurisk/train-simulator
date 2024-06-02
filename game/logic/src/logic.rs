use log::info;
use shared_domain::game_state::GameState;
use shared_domain::map_level::MapLevel;
use shared_domain::{
    BuildingId, BuildingInfo, BuildingType, GameId, PlayerId, PlayerName, TrackType,
};
use shared_protocol::client_command::{ClientCommand, LobbyCommand};
use shared_protocol::server_response::{
    AddressEnvelope, GameInfo, GameResponse, LobbyResponse, ServerResponse,
    ServerResponseWithAddress,
};
use shared_util::coords_xz::CoordsXZ;

// TODO: Should be more stateful and separated into Authentication, Lobby, Game
#[allow(clippy::module_name_repetitions, clippy::missing_panics_doc)]
#[must_use]
pub fn server_logic(
    client_command: &ClientCommand,
    player_id: PlayerId,
) -> Vec<ServerResponseWithAddress> {
    match client_command {
        ClientCommand::Lobby(lobby_command) => {
            match lobby_command {
                LobbyCommand::CreateGame => {
                    let game_id = GameId::random();
                    let level_json = include_str!("../assets/map_levels/default.json");
                    let level = serde_json::from_str::<MapLevel>(level_json)
                        .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));

                    assert!(level.is_valid());

                    let game_state = GameState { map_level: level };
                    let players = vec![(player_id, PlayerName::random())]
                        .into_iter()
                        .collect();

                    info!("Simulating server responding to JoinGame with GameJoined");

                    vec![
                        ServerResponseWithAddress::new(
                            AddressEnvelope::ToAllPlayersInGame(game_id),
                            ServerResponse::Lobby(LobbyResponse::GameJoined(GameInfo {
                                game_id,
                                players,
                            })),
                        ),
                        ServerResponseWithAddress::new(
                            AddressEnvelope::ToPlayer(player_id),
                            ServerResponse::Game(GameResponse::State(game_state)),
                        ),
                        ServerResponseWithAddress::new(
                            AddressEnvelope::ToAllPlayersInGame(game_id),
                            ServerResponse::Game(GameResponse::BuildingBuilt(BuildingInfo {
                                building_id:      BuildingId::random(),
                                vertex_coords_xz: CoordsXZ::new(3, 5),
                                building_type:    BuildingType::Track(TrackType::NorthSouth),
                            })),
                        ),
                    ]
                },
                _ => todo!(), // TODO: Implement other handling
            }
        },
        _ => todo!(), // TODO: Implement other handling
    }
}
