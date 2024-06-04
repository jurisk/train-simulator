#![allow(
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::unnecessary_wraps
)]

use std::collections::HashMap;
use std::convert::identity;

use log::warn;
use shared_domain::client_command::{ClientCommand, ClientCommandWithClientId};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{
    AddressEnvelope, ServerResponse, ServerResponseWithAddress, ServerResponseWithClientIds,
};
use shared_domain::{
    BuildingId, BuildingInfo, BuildingType, ClientId, GameId, PlayerId, TrackType,
};
use shared_util::coords_xz::CoordsXZ;

use crate::authentication_logic::{lookup_player_id, process_authentication_command};
use crate::connection_registry::ConnectionRegistry;
use crate::game_logic::{lookup_game_state, process_game_command};
use crate::game_state::GameState;
use crate::lobby_logic::process_lobby_command;

pub struct ServerState {
    pub connection_registry: ConnectionRegistry,
    pub games:               HashMap<GameId, GameState>,

    game_prototype: GameState,
}

impl ServerState {
    #[must_use]
    #[allow(clippy::missing_panics_doc, clippy::new_without_default)]
    pub fn new() -> Self {
        let level_json = include_str!("../assets/map_levels/default.json");
        let default_level = serde_json::from_str::<MapLevel>(level_json)
            .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));
        assert!(default_level.is_valid());

        let initial_buildings = vec![
            BuildingInfo {
                building_id:          BuildingId::random(),
                north_west_vertex_xz: CoordsXZ::new(10, 10),
                building_type:        BuildingType::Track(TrackType::EastWest),
            },
            BuildingInfo {
                building_id:          BuildingId::random(),
                north_west_vertex_xz: CoordsXZ::new(3, 5),
                building_type:        BuildingType::Track(TrackType::NorthSouth),
            },
        ];

        let game_prototype = GameState {
            map_level: default_level,
            buildings: initial_buildings,
            players:   HashMap::new(),
        };

        Self {
            connection_registry: ConnectionRegistry::new(),
            games: HashMap::new(),
            game_prototype,
        }
    }

    fn client_ids_for_player(&self, player_id: PlayerId) -> Vec<ClientId> {
        match self.connection_registry.get_client_id(&player_id) {
            None => {
                warn!("Failed to find client_id for {player_id:?}");
                vec![]
            },
            Some(client_id) => vec![*client_id],
        }
    }

    fn translate_response(
        &self,
        server_response_with_address: ServerResponseWithAddress,
    ) -> ServerResponseWithClientIds {
        let client_ids = match server_response_with_address.address {
            AddressEnvelope::ToClient(client_id) => vec![client_id],
            AddressEnvelope::ToPlayer(player_id) => self.client_ids_for_player(player_id),
            AddressEnvelope::ToAllPlayersInGame(game_id) => {
                let player_ids = match self.games.get(&game_id) {
                    None => {
                        warn!("Failed to find game for {game_id:?}");
                        vec![]
                    },
                    Some(game_state) => game_state.players.keys().copied().collect(),
                };

                player_ids
                    .into_iter()
                    .flat_map(|player_id| self.client_ids_for_player(player_id))
                    .collect()
            },
        };

        ServerResponseWithClientIds {
            client_ids,
            response: server_response_with_address.response,
        }
    }

    fn process_internal(
        &mut self,
        client_command_with_client_id: ClientCommandWithClientId,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        let client_id = client_command_with_client_id.client_id;
        match client_command_with_client_id.command {
            ClientCommand::Authentication(authentication_command) => {
                process_authentication_command(
                    &mut self.connection_registry,
                    client_id,
                    authentication_command,
                )
            },
            ClientCommand::Lobby(lobby_command) => {
                let requesting_player_id = lookup_player_id(&self.connection_registry, client_id)?;
                process_lobby_command(
                    &mut self.games,
                    requesting_player_id,
                    lobby_command,
                    &self.game_prototype,
                )
            },
            ClientCommand::Game(game_id, game_command) => {
                let requesting_player_id = lookup_player_id(&self.connection_registry, client_id)?;
                let game_state = lookup_game_state(&mut self.games, game_id)?;
                process_game_command(game_id, game_state, requesting_player_id, game_command)
            },
        }
    }

    #[must_use]
    pub fn process(
        &mut self,
        client_command_with_client_id: ClientCommandWithClientId,
    ) -> Vec<ServerResponseWithClientIds> {
        let client_id = client_command_with_client_id.client_id;
        let responses = self.process_internal(client_command_with_client_id);

        let flattened = responses
            .map_err(|response| {
                vec![ServerResponseWithAddress {
                    address: AddressEnvelope::ToClient(client_id),
                    response,
                }]
            })
            .unwrap_or_else(identity);

        flattened
            .into_iter()
            .map(|response| self.translate_response(response))
            .collect()
    }
}
