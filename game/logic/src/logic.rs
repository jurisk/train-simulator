use log::info;
use shared_domain::game_state::GameState;
use shared_domain::map_level::MapLevel;
use shared_domain::{
    BuildingId, BuildingInfo, BuildingType, GameId, PlayerId, PlayerName, TrackType,
};
use shared_protocol::client_command::{
    AuthenticationCommand, ClientCommand, ClientCommandWithClientId, GameCommand, LobbyCommand,
};
use shared_protocol::server_response::{
    AddressEnvelope, AuthenticationResponse, GameInfo, GameResponse, LobbyResponse, ServerResponse,
    ServerResponseWithAddress,
};
use shared_util::coords_xz::CoordsXZ;

// TODO: Should be more stateful and separated into Authentication, Lobby, Game
#[allow(clippy::module_name_repetitions, clippy::missing_panics_doc)]
#[must_use]
pub fn server_logic(
    client_command_with_client_id: ClientCommandWithClientId,
) -> Vec<ServerResponseWithAddress> {
    let client_id = client_command_with_client_id.client_id;
    match client_command_with_client_id.command {
        ClientCommand::Authentication(authentication_command) => {
            match authentication_command {
                AuthenticationCommand::Login(player_id, access_token) => {
                    if access_token.0 == "valid-token" {
                        // TODO: Update map between PlayerId and ClientId
                        vec![ServerResponseWithAddress::new(
                            AddressEnvelope::ToClient(client_id),
                            ServerResponse::Authentication(AuthenticationResponse::LoginSucceeded(
                                player_id,
                            )),
                        )]
                    } else {
                        vec![ServerResponseWithAddress::new(
                            AddressEnvelope::ToClient(client_id),
                            ServerResponse::Authentication(AuthenticationResponse::LoginFailed),
                        )]
                    }
                },
                AuthenticationCommand::Logout => {
                    vec![]
                },
            }
        },
        ClientCommand::Lobby(lobby_command) => {
            match lobby_command {
                LobbyCommand::CreateGame => {
                    let game_id = GameId::random();
                    let level_json = include_str!("../assets/map_levels/default.json");
                    let map_level = serde_json::from_str::<MapLevel>(level_json)
                        .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));

                    assert!(map_level.is_valid());

                    let buildings = vec![BuildingInfo {
                        building_id:          BuildingId::random(),
                        north_west_vertex_xz: CoordsXZ::new(10, 10),
                        building_type:        BuildingType::Track(TrackType::EastWest),
                    }];

                    let game_state = GameState {
                        map_level,
                        buildings,
                    };
                    let player_id = PlayerId::random();
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
                                building_id:          BuildingId::random(),
                                north_west_vertex_xz: CoordsXZ::new(3, 5),
                                building_type:        BuildingType::Track(TrackType::NorthSouth),
                            })),
                        ),
                    ]
                },
                _ => todo!(), // TODO: Implement other handling
            }
        },
        ClientCommand::Game(game_command) => {
            match game_command {
                GameCommand::BuildBuilding(building_info) => {
                    let game_id = GameId::random(); // TODO: Actually detect which game player is in
                    // TODO: Check that you can build there
                    // TODO: Update game state with the buildings
                    vec![ServerResponseWithAddress::new(
                        AddressEnvelope::ToAllPlayersInGame(game_id),
                        ServerResponse::Game(GameResponse::BuildingBuilt(building_info.clone())),
                    )]
                },
            }
        },
    }
}
