#![allow(clippy::missing_errors_doc, clippy::unnecessary_wraps)]

use std::collections::HashMap;

use log::warn;
use shared_domain::client_command::{GameCommand, LobbyCommand};
use shared_domain::game_state::GameState;
use shared_domain::game_time::GameTimeDiff;
use shared_domain::metrics::Metrics;
use shared_domain::scenario::{EUROPE_SCENARIO_BINCODE, Scenario, USA_SCENARIO_BINCODE};
use shared_domain::server_response::{
    AddressEnvelope, GameError, GameResponse, LobbyResponse, ServerError, ServerResponse,
    ServerResponseWithAddress, UserInfo,
};
use shared_domain::{GameId, PlayerId, ScenarioId, UserId};
use shared_util::compression::load_from_bytes;

use crate::game_service::{GameResponseWithAddress, GameService};

// This is also, in a way, `Lobby`. Should we rename it? Split into two somehow? Not sure yet...
pub struct GamesService {
    game_map:                    HashMap<GameId, GameService>,
    game_scenarios:              HashMap<ScenarioId, Scenario>,
    ignore_requesting_player_id: bool,
}

impl GamesService {
    #[must_use]
    #[expect(clippy::match_same_arms, clippy::missing_panics_doc)]
    pub fn new(ignore_requesting_player_id: bool) -> Self {
        let mut game_scenarios = HashMap::new();
        for scenario_id in ScenarioId::all() {
            let ScenarioId(scenario_name) = &scenario_id;
            let scenario_bincode = match scenario_name.as_str() {
                "europe" => EUROPE_SCENARIO_BINCODE,
                "usa_east" => USA_SCENARIO_BINCODE,
                _ => USA_SCENARIO_BINCODE,
            };
            let scenario: Scenario = load_from_bytes(scenario_bincode)
                .unwrap_or_else(|err| panic!("Failed to load scenario {scenario_id:?}: {err}"));

            assert!(scenario.is_valid().is_ok());

            assert_eq!(scenario.scenario_id, scenario_id);

            game_scenarios.insert(scenario_id, scenario);
        }

        Self {
            game_map: HashMap::new(),
            game_scenarios,
            ignore_requesting_player_id,
        }
    }

    pub(crate) fn user_ids_for_player(&self, game_id: GameId, player_id: PlayerId) -> Vec<UserId> {
        self.game_map
            .get(&game_id)
            .map(|game_service| game_service.user_ids_for_player(player_id))
            .unwrap_or_default()
    }

    pub fn advance_time_diffs(&mut self, diff: GameTimeDiff, metrics: &impl Metrics) {
        for game_service in self.game_map.values_mut() {
            game_service.advance_time_diff(diff, metrics);
        }
    }

    pub(crate) fn sync_games(&mut self) -> Vec<ServerResponseWithAddress> {
        self.game_map
            .iter_mut()
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
        requesting_user_id: UserId,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        let game_infos = self
            .game_map
            .values()
            .map(GameService::create_game_info)
            .collect();
        Ok(vec![ServerResponseWithAddress::new(
            AddressEnvelope::ToUser(requesting_user_id),
            ServerResponse::Lobby(LobbyResponse::AvailableGames(game_infos)),
        )])
    }

    pub fn create_and_join_game_by_game_state(
        &mut self,
        requesting_user_info: &UserInfo,
        game_state: GameState,
        player_id: Option<PlayerId>,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        let game_service = GameService::from_game_state(game_state);
        self.join_and_insert_game(game_service, requesting_user_info, player_id)
    }

    pub fn create_and_join_game_by_scenario(
        &mut self,
        requesting_user_info: &UserInfo,
        scenario_id: &ScenarioId,
        player_id: Option<PlayerId>,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        // Later: Don't allow starting a game if is already a part of another game?
        let scenario = self.game_scenarios.get(scenario_id).ok_or_else(|| {
            Box::new(ServerResponse::Error(ServerError::ScenarioNotFound(
                scenario_id.clone(),
            )))
        })?;

        let game_service = GameService::from_prototype(scenario, self.ignore_requesting_player_id);

        self.join_and_insert_game(game_service, requesting_user_info, player_id)
    }

    fn join_and_insert_game(
        &mut self,
        mut game_service: GameService,
        user_info: &UserInfo,
        player_id: Option<PlayerId>,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        let game_id = game_service.game_id();

        let response = game_service.join_game(user_info, player_id);

        let results = Self::convert_game_response_to_server_response(game_id, response)?;
        self.game_map.insert(game_id, game_service);
        Ok(results)
    }

    pub fn join_game(
        &mut self,
        user_info: &UserInfo,
        game_id: GameId,
        player_id: Option<PlayerId>,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        let game_service = self.lookup_game_service_mut(game_id)?;
        Self::convert_game_response_to_server_response(
            game_id,
            game_service.join_game(user_info, player_id),
        )
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
        requesting_user_id: UserId,
        game_command: &GameCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        let game_service = self.lookup_game_service_mut(game_id)?;
        let player_id = game_service
            .player_id_for_user_id(requesting_user_id)
            .ok_or(ServerResponse::Game(
                game_id,
                GameResponse::Error(GameError::UnspecifiedError),
            ))?;
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
        user_info: &UserInfo,
        lobby_command: &LobbyCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        match lobby_command {
            LobbyCommand::ListGames => self.create_game_infos(user_info.id),
            LobbyCommand::CreateAndJoinGameByScenario(scenario_id, player_id) => {
                self.create_and_join_game_by_scenario(user_info, scenario_id, *player_id)
            },
            LobbyCommand::CreateAndJoinGameByGameState(game_state, player_id) => {
                self.create_and_join_game_by_game_state(
                    user_info,
                    game_state.as_ref().clone(),
                    *player_id,
                )
            },
            LobbyCommand::JoinExistingGame(game_id, player_id) => {
                self.join_game(user_info, *game_id, *player_id)
            },
            LobbyCommand::LeaveGame(game_id) => {
                // Later: Not sure how this should even work if the player has buildings and transport owned in the game?
                let game_service = self.lookup_game_service_mut(*game_id)?;
                Self::convert_game_response_to_server_response(
                    *game_id,
                    game_service.remove_player(user_info.id),
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

    // Public as a hack for testing only
    pub fn get_game_service_mut(&mut self, game_id: GameId) -> Option<&mut GameService> {
        self.game_map.get_mut(&game_id)
    }
}
