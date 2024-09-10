#![allow(clippy::missing_errors_doc, clippy::unnecessary_wraps)]

use std::collections::HashMap;

use log::warn;
use shared_domain::client_command::{GameCommand, LobbyCommand};
use shared_domain::game_state::GameState;
use shared_domain::game_time::GameTime;
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::metrics::Metrics;
use shared_domain::server_response::{
    AddressEnvelope, GameError, GameResponse, LobbyResponse, PlayerInfo, ServerError,
    ServerResponse, ServerResponseWithAddress,
};
use shared_domain::{GameId, MapId, PlayerId};

use crate::game_service::{GameResponseWithAddress, GameService};

// This is also, in a way, `Lobby`. Should we rename it? Split into two somehow? Not sure yet...
pub struct GamesService {
    game_map:        HashMap<GameId, GameService>,
    game_prototypes: HashMap<MapId, GameState>,
}

impl GamesService {
    #[must_use]
    #[expect(clippy::new_without_default, clippy::match_same_arms)]
    pub fn new() -> Self {
        // Later: Eventually, eliminate the Sample map level
        const EUROPE_LEVEL_JSON: &str = include_str!("../../../assets/map_levels/europe.json");
        // TODO: Have a full USA map and use that
        const USA_LEVEL_JSON: &str = include_str!("../../../assets/map_levels/usa_east.json");

        let mut game_prototypes = HashMap::new();
        for map_id in MapId::all() {
            let MapId(map_name) = &map_id;
            let level_json = match map_name.as_str() {
                "europe" => EUROPE_LEVEL_JSON,
                "usa_east" => USA_LEVEL_JSON,
                _ => USA_LEVEL_JSON,
            };
            let map_level = MapLevel::load(level_json);
            let game_prototype = GameState::empty_from_level(map_id.clone(), map_level);

            game_prototypes.insert(map_id, game_prototype.clone());
        }

        Self {
            game_map: HashMap::new(),
            game_prototypes,
        }
    }

    pub fn advance_times(&mut self, time: GameTime, metrics: &impl Metrics) {
        for game_service in self.game_map.values_mut() {
            game_service.advance_time(time, metrics);
        }
    }

    pub(crate) fn sync_games(&self) -> Vec<ServerResponseWithAddress> {
        self.game_map
            .iter()
            .flat_map(|(game_id, game_service)| {
                let results = game_service.sync();
                results
                    .into_iter()
                    .map(|game_response| {
                        ServerResponseWithAddress::new(
                            game_response.address,
                            ServerResponse::Game(*game_id, game_response.response),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn create_game_infos(
        &self,
        requesting_player_id: PlayerId,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        let game_infos = self
            .game_map
            .values()
            .map(GameService::create_game_info)
            .collect();
        Ok(vec![ServerResponseWithAddress::new(
            AddressEnvelope::ToPlayer(requesting_player_id),
            ServerResponse::Lobby(LobbyResponse::AvailableGames(game_infos)),
        )])
    }

    pub fn create_and_join_game(
        &mut self,
        requesting_player_info: &PlayerInfo,
        map_id: &MapId,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        // Later: Don't allow starting a game if is already a part of another game?
        let prototype = self.game_prototypes.get(map_id).ok_or_else(|| {
            Box::new(ServerResponse::Error(ServerError::MapNotFound(
                map_id.clone(),
            )))
        })?;
        let mut game_service = GameService::from_prototype(prototype);
        let game_id = game_service.game_id();
        let results = game_service
            .join_game(requesting_player_info)
            .map_err(|err| ServerResponse::Game(game_id, GameResponse::Error(err)))?;
        self.game_map.insert(game_id, game_service);
        Self::convert_game_response_to_server_response(game_id, Ok(results))
    }

    fn convert_game_response_to_server_response(
        game_id: GameId,
        input: Result<Vec<GameResponseWithAddress>, GameError>,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
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
            Err(game_response) => {
                Err(Box::new(ServerResponse::Game(
                    game_id,
                    GameResponse::Error(game_response),
                )))
            },
        }
    }

    pub fn process_command(
        &mut self,
        game_id: GameId,
        player_id: PlayerId,
        game_command: &GameCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        let game_service = self.lookup_game_service_mut(game_id)?;
        Self::convert_game_response_to_server_response(
            game_id,
            game_service.process_command(player_id, game_command),
        )
    }

    fn lookup_game_service_mut(
        &mut self,
        game_id: GameId,
    ) -> Result<&mut GameService, Box<ServerResponse>> {
        match self.game_map.get_mut(&game_id) {
            None => {
                Err(Box::new(ServerResponse::Game(
                    game_id,
                    GameResponse::Error(GameError::GameNotFound),
                )))
            },
            Some(result) => Ok(result),
        }
    }

    fn lookup_game_service(&self, game_id: GameId) -> Result<&GameService, Box<ServerResponse>> {
        match self.game_map.get(&game_id) {
            None => {
                Err(Box::new(ServerResponse::Game(
                    game_id,
                    GameResponse::Error(GameError::GameNotFound),
                )))
            },
            Some(result) => Ok(result),
        }
    }

    pub(crate) fn process_lobby_command(
        &mut self,
        player_info: &PlayerInfo,
        lobby_command: &LobbyCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        match lobby_command {
            LobbyCommand::ListGames => self.create_game_infos(player_info.id),
            LobbyCommand::CreateGame(map_id) => self.create_and_join_game(player_info, map_id),
            LobbyCommand::JoinExistingGame(game_id) => {
                let game_service = self.lookup_game_service_mut(*game_id)?;
                Self::convert_game_response_to_server_response(
                    *game_id,
                    game_service.join_game(player_info),
                )
            },
            LobbyCommand::LeaveGame(game_id) => {
                // Later: Not sure how this should even work if the player has buildings and transport owned in the game?
                let game_service = self.lookup_game_service_mut(*game_id)?;
                Self::convert_game_response_to_server_response(
                    *game_id,
                    game_service.remove_player(player_info.id),
                )
            },
        }
    }

    #[expect(clippy::single_match_else)]
    pub(crate) fn players_in_game(&self, game_id: GameId) -> Vec<PlayerId> {
        match self.lookup_game_service(game_id) {
            Ok(found) => found.player_ids(),
            Err(_) => {
                warn!("Failed to find game for {game_id:?}");
                vec![]
            },
        }
    }
}
