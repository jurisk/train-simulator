#![allow(clippy::missing_errors_doc, clippy::unnecessary_wraps)]

use std::collections::HashMap;

use log::warn;
use shared_domain::client_command::{GameCommand, LobbyCommand};
use shared_domain::game_time::GameTime;
use shared_domain::metrics::Metrics;
use shared_domain::scenario::{EUROPE_SCENARIO_BINCODE, Scenario, USA_SCENARIO_BINCODE};
use shared_domain::server_response::{
    AddressEnvelope, GameError, GameResponse, LobbyResponse, ServerError, ServerResponse,
    ServerResponseWithAddress, UserInfo,
};
use shared_domain::{GameId, PlayerId, ScenarioId, UserId};

use crate::game_service::{GameResponseWithAddress, GameService};

// This is also, in a way, `Lobby`. Should we rename it? Split into two somehow? Not sure yet...
pub struct GamesService {
    game_map:       HashMap<GameId, GameService>,
    game_scenarios: HashMap<ScenarioId, Scenario>,
}

impl GamesService {
    #[must_use]
    #[expect(
        clippy::new_without_default,
        clippy::match_same_arms,
        clippy::missing_panics_doc
    )]
    pub fn new() -> Self {
        let mut game_scenarios = HashMap::new();
        for scenario_id in ScenarioId::all() {
            let ScenarioId(scenario_name) = &scenario_id;
            let scenario_bincode = match scenario_name.as_str() {
                "europe" => EUROPE_SCENARIO_BINCODE,
                "usa_east" => USA_SCENARIO_BINCODE,
                _ => USA_SCENARIO_BINCODE,
            };
            let scenario = Scenario::load_from_bytes(scenario_bincode)
                .unwrap_or_else(|err| panic!("Failed to load scenario {scenario_id:?}: {err}"));

            assert_eq!(scenario.scenario_id, scenario_id);

            game_scenarios.insert(scenario_id, scenario);
        }

        Self {
            game_map: HashMap::new(),
            game_scenarios,
        }
    }

    pub(crate) fn user_ids_for_player(&self, game_id: GameId, player_id: PlayerId) -> Vec<UserId> {
        self.game_map
            .get(&game_id)
            .map(|game_service| game_service.user_ids_for_player(player_id))
            .unwrap_or_default()
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

    pub fn create_and_join_game(
        &mut self,
        requesting_user_info: &UserInfo,
        scenario_id: &ScenarioId,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        // Later: Don't allow starting a game if is already a part of another game?
        let scenario = self.game_scenarios.get(scenario_id).ok_or_else(|| {
            Box::new(ServerResponse::Error(ServerError::ScenarioNotFound(
                scenario_id.clone(),
            )))
        })?;

        let mut game_service = GameService::from_prototype(scenario);
        let game_id = game_service.game_id();

        // Later: Allow picking a particular `player_id` to be chosen
        // let player_id = prototype.players().ids().first().copied();
        // let player_id = player_id.ok_or(ServerResponse::Game(game_id, GameResponse::Error(GameError::UnspecifiedError)))?;

        let results = game_service
            .join_game(requesting_user_info)
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
            LobbyCommand::CreateGame(scenario_id) => {
                self.create_and_join_game(user_info, scenario_id)
            },
            LobbyCommand::JoinExistingGame(game_id) => {
                let game_service = self.lookup_game_service_mut(*game_id)?;
                Self::convert_game_response_to_server_response(
                    *game_id,
                    game_service.join_game(user_info),
                )
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
}
