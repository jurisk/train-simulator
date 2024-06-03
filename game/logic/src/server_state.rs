use std::collections::HashMap;

use log::info;
use shared_domain::client_command::{
    AuthenticationCommand, ClientCommand, ClientCommandWithClientId, GameCommand, LobbyCommand,
};
use shared_domain::game_state::GameState;
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{
    AuthenticationResponse, GameInfo, GameResponse, LobbyResponse, ServerResponse,
    ServerResponseWithClientIds,
};
use shared_domain::{
    BuildingId, BuildingInfo, BuildingType, GameId, PlayerId, PlayerName, TrackType,
};
use shared_util::coords_xz::CoordsXZ;

use crate::connection_registry::ConnectionRegistry;

#[derive(Default)]
pub struct ServerState {
    pub connection_registry: ConnectionRegistry,
    pub games:               HashMap<GameId, GameState>,
}

impl ServerState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            connection_registry: ConnectionRegistry::new(),
            games:               HashMap::new(),
        }
    }

    // TODO: Should be separated into Authentication, Lobby, Game processing, some of them returning ServerResponseWithAddress
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn process(
        &mut self,
        client_command_with_client_id: ClientCommandWithClientId,
    ) -> Vec<ServerResponseWithClientIds> {
        let client_id = client_command_with_client_id.client_id;
        match client_command_with_client_id.command {
            ClientCommand::Authentication(authentication_command) => {
                match authentication_command {
                    AuthenticationCommand::Login(player_id, access_token) => {
                        if access_token.0 == "valid-token" {
                            // TODO: Update map between PlayerId and ClientId
                            vec![ServerResponseWithClientIds::new(
                                vec![client_id],
                                ServerResponse::Authentication(
                                    AuthenticationResponse::LoginSucceeded(player_id),
                                ),
                            )]
                        } else {
                            vec![ServerResponseWithClientIds::new(
                                vec![client_id],
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
                            .unwrap_or_else(|err| {
                                panic!("Failed to deserialise {level_json}: {err}")
                            });

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
                            ServerResponseWithClientIds::new(
                                vec![client_id], // TODO: Actually all players in game
                                ServerResponse::Lobby(LobbyResponse::GameJoined(GameInfo {
                                    game_id,
                                    players,
                                })),
                            ),
                            ServerResponseWithClientIds::new(
                                vec![client_id],
                                ServerResponse::Game(GameResponse::State(game_state)),
                            ),
                            ServerResponseWithClientIds::new(
                                vec![client_id], // TODO: Actually all players in game
                                ServerResponse::Game(GameResponse::BuildingBuilt(BuildingInfo {
                                    building_id:          BuildingId::random(),
                                    north_west_vertex_xz: CoordsXZ::new(3, 5),
                                    building_type:        BuildingType::Track(
                                        TrackType::NorthSouth,
                                    ),
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
                        // TODO: Check that you can build there
                        // TODO: Update game state with the buildings
                        vec![ServerResponseWithClientIds::new(
                            vec![client_id], // TODO: Actually all players in game
                            ServerResponse::Game(GameResponse::BuildingBuilt(
                                building_info.clone(),
                            )),
                        )]
                    },
                }
            },
        }
    }
}
