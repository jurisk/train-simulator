#![allow(clippy::missing_errors_doc, clippy::unnecessary_wraps)]

use std::collections::HashMap;

use log::warn;
use shared_domain::client_command::{GameCommand, LobbyCommand};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{
    AddressEnvelope, GameResponse, LobbyResponse, ServerError, ServerResponse,
    ServerResponseWithAddress,
};
use shared_domain::{GameId, PlayerId};

use crate::game_state::{GameResponseWithAddress, GameState, GameTime};

// This is also, in a way, `Lobby`. Should we rename it? Split into two somehow? Not sure yet...
pub(crate) struct Games {
    game_map:       HashMap<GameId, GameState>,
    game_prototype: GameState,
}

impl Games {
    #[must_use]
    #[allow(clippy::missing_panics_doc, clippy::new_without_default)]
    pub(crate) fn new() -> Self {
        let level_json = include_str!("../../../assets/map_levels/default.json");
        let default_level = serde_json::from_str::<MapLevel>(level_json)
            .unwrap_or_else(|err| panic!("Failed to deserialise {level_json}: {err}"));
        assert!(default_level.is_valid());

        let game_prototype = GameState::new(default_level, vec![], vec![], HashMap::new());

        Self {
            game_map: HashMap::new(),
            game_prototype,
        }
    }

    pub(crate) fn advance_times(&mut self, time: GameTime) {
        for game_state in self.game_map.values_mut() {
            game_state.advance_time(time);
        }
    }

    fn create_game_infos(
        &self,
        requesting_player_id: PlayerId,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        let game_infos = self
            .game_map
            .values()
            .map(GameState::create_game_info)
            .collect();
        Ok(vec![ServerResponseWithAddress::new(
            AddressEnvelope::ToPlayer(requesting_player_id),
            ServerResponse::Lobby(LobbyResponse::AvailableGames(game_infos)),
        )])
    }

    pub(crate) fn create_and_join_game(
        &mut self,
        requesting_player_id: PlayerId,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        // Later: Don't allow starting a game if is already a part of another game?
        let mut game_state = GameState::from_prototype(&self.game_prototype);
        let game_id = game_state.game_id;
        let results = game_state
            .join_game(requesting_player_id)
            .map_err(|err| ServerResponse::Game(game_id, err))?;
        self.game_map.insert(game_id, game_state);
        Self::convert_game_response_to_server_response(game_id, Ok(results))
    }

    fn convert_game_response_to_server_response(
        game_id: GameId,
        input: Result<Vec<GameResponseWithAddress>, GameResponse>,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        match input {
            Ok(game_responses) => {
                Ok(game_responses
                    .into_iter()
                    .map(|game_response| {
                        ServerResponseWithAddress::new(
                            game_response.address,
                            ServerResponse::Game(game_id, game_response.response),
                        )
                    })
                    .collect())
            },
            Err(game_response) => Err(ServerResponse::Game(game_id, game_response)),
        }
    }

    pub(crate) fn process_command(
        &mut self,
        game_id: GameId,
        player_id: PlayerId,
        game_command: GameCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        let game_state = self.lookup_game_state_mut(game_id)?;
        Self::convert_game_response_to_server_response(
            game_id,
            game_state.process_command(player_id, game_command),
        )
    }

    fn lookup_game_state_mut(&mut self, game_id: GameId) -> Result<&mut GameState, ServerResponse> {
        match self.game_map.get_mut(&game_id) {
            None => Err(ServerResponse::Error(ServerError::GameNotFound)),
            Some(result) => Ok(result),
        }
    }

    fn lookup_game_state(&self, game_id: GameId) -> Result<&GameState, ServerResponse> {
        match self.game_map.get(&game_id) {
            None => Err(ServerResponse::Error(ServerError::GameNotFound)),
            Some(result) => Ok(result),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn process_lobby_command(
        &mut self,
        requesting_player_id: PlayerId,
        lobby_command: LobbyCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        match lobby_command {
            LobbyCommand::ListGames => self.create_game_infos(requesting_player_id),
            LobbyCommand::CreateGame => self.create_and_join_game(requesting_player_id),
            LobbyCommand::JoinExistingGame(game_id) => {
                let game_state = self.lookup_game_state_mut(game_id)?;
                Self::convert_game_response_to_server_response(
                    game_id,
                    game_state.join_game(requesting_player_id),
                )
            },
            LobbyCommand::LeaveGame(game_id) => {
                // Later: Not sure how this should even work if the player has buildings and transport owned in the game?
                let game_state = self.lookup_game_state_mut(game_id)?;
                Self::convert_game_response_to_server_response(
                    game_id,
                    game_state.remove_player(requesting_player_id),
                )
            },
        }
    }

    #[allow(clippy::single_match_else)]
    pub(crate) fn players_in_game(&self, game_id: GameId) -> Vec<PlayerId> {
        match self.lookup_game_state(game_id) {
            Ok(found) => found.player_ids(),
            Err(_) => {
                warn!("Failed to find game for {game_id:?}");
                vec![]
            },
        }
    }
}
