#![allow(clippy::missing_errors_doc)]

use std::collections::HashMap;

use log::warn;
use shared_domain::client_command::LobbyCommand;
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{
    AddressEnvelope, GameInfo, LobbyResponse, ServerError, ServerResponse,
    ServerResponseWithAddress,
};
use shared_domain::{
    BuildingId, BuildingInfo, BuildingType, GameId, PlayerId, PlayerName, TrackType,
};
use shared_util::coords_xz::CoordsXZ;

use crate::game_state::GameState;

pub struct Games {
    game_map:       HashMap<GameId, GameState>,
    game_prototype: GameState,
}

// TODO: Make the public API smaller
impl Games {
    #[must_use]
    #[allow(clippy::missing_panics_doc, clippy::new_without_default)]
    pub(crate) fn new() -> Self {
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
            game_map: HashMap::new(),
            game_prototype,
        }
    }

    #[must_use]
    pub(crate) fn create_game_infos(&self) -> Vec<GameInfo> {
        self.game_map
            .iter()
            .map(|(game_id, game_state)| {
                GameInfo {
                    game_id: *game_id,
                    players: game_state.players.clone(),
                }
            })
            .collect()
    }

    pub(crate) fn create_and_join_game(
        &mut self,
        requesting_player_id: PlayerId,
        requesting_player_name: PlayerName,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        // Later: Don't allow starting a game if is already a part of another game?

        let game_id = GameId::random();

        let mut game_state = self.game_prototype.clone();

        let results =
            game_state.join_game(game_id, requesting_player_id, requesting_player_name)?;

        self.game_map.insert(game_id, game_state);

        Ok(results)
    }

    pub(crate) fn lookup_game_state_mut(
        &mut self,
        game_id: GameId,
    ) -> Result<&mut GameState, ServerResponse> {
        match self.game_map.get_mut(&game_id) {
            None => Err(ServerResponse::Error(ServerError::GameNotFound)),
            Some(result) => Ok(result),
        }
    }

    pub(crate) fn lookup_game_state(&self, game_id: GameId) -> Result<&GameState, ServerResponse> {
        match self.game_map.get(&game_id) {
            None => Err(ServerResponse::Error(ServerError::GameNotFound)),
            Some(result) => Ok(result),
        }
    }

    pub(crate) fn process_lobby_command(
        &mut self,
        requesting_player_id: PlayerId,
        lobby_command: LobbyCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        match lobby_command {
            LobbyCommand::ListGames => {
                Ok(vec![ServerResponseWithAddress::new(
                    AddressEnvelope::ToPlayer(requesting_player_id),
                    ServerResponse::Lobby(LobbyResponse::AvailableGames(self.create_game_infos())),
                )])
            },
            LobbyCommand::CreateGame(player_name) => {
                self.create_and_join_game(requesting_player_id, player_name)
            },
            LobbyCommand::JoinExistingGame(game_id, player_name) => {
                let game_state = self.lookup_game_state_mut(game_id)?;
                game_state.join_game(game_id, requesting_player_id, player_name)
            },
            LobbyCommand::LeaveGame(game_id) => {
                // Later: Not sure how this should even work if the player has buildings and vehicles owned in the game?
                let game_state = self.lookup_game_state_mut(game_id)?;
                game_state.players.remove(&requesting_player_id);
                Ok(vec![ServerResponseWithAddress::new(
                    AddressEnvelope::ToAllPlayersInGame(game_id),
                    ServerResponse::Lobby(LobbyResponse::GameLeft(game_id)),
                )])
            },
        }
    }

    #[allow(clippy::single_match_else)]
    pub(crate) fn players_in_game(&self, game_id: GameId) -> Vec<PlayerId> {
        match self.lookup_game_state(game_id) {
            Ok(found) => found.players.keys().copied().collect(),
            Err(_) => {
                warn!("Failed to find game for {game_id:?}");
                vec![]
            },
        }
    }
}
