#![allow(
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::unnecessary_wraps
)]

use std::convert::identity;

use shared_domain::client_command::{ClientCommand, ClientCommandWithClientId, NetworkCommand};
use shared_domain::game_time::GameTimeDiff;
use shared_domain::metrics::Metrics;
use shared_domain::server_response::{
    AddressEnvelope, NetworkResponse, ServerResponse, ServerResponseWithAddress,
    ServerResponseWithClientIds,
};
use shared_domain::{ClientId, GameId, PlayerId, UserId};

use crate::authentication_service::AuthenticationService;
use crate::games_service::GamesService;

pub struct ServerState {
    authentication_service: AuthenticationService,
    games_service:          GamesService,
}

impl ServerState {
    #[must_use]
    pub fn new(ignore_requesting_player_id: bool) -> Self {
        Self {
            authentication_service: AuthenticationService::new(),
            games_service:          GamesService::new(ignore_requesting_player_id),
        }
    }

    pub fn advance_time_diffs(&mut self, diff: GameTimeDiff, metrics: &impl Metrics) {
        self.games_service.advance_time_diffs(diff, metrics);
    }

    #[must_use]
    pub fn sync_games(&self) -> Vec<ServerResponseWithClientIds> {
        self.games_service
            .sync_games()
            .into_iter()
            .map(|response| self.translate_response(response))
            .collect()
    }

    fn user_ids_for_player(&self, game_id: GameId, player_id: PlayerId) -> Vec<UserId> {
        self.games_service.user_ids_for_player(game_id, player_id)
    }

    fn translate_response(
        &self,
        server_response_with_address: ServerResponseWithAddress,
    ) -> ServerResponseWithClientIds {
        let client_ids = match server_response_with_address.address {
            AddressEnvelope::ToClient(client_id) => vec![client_id],
            AddressEnvelope::ToPlayer(game_id, player_id) => {
                self.user_ids_for_player(game_id, player_id)
                    .into_iter()
                    .flat_map(|user_id| self.authentication_service.client_ids_for_user(user_id))
                    .collect()
            },
            AddressEnvelope::ToUser(user_id) => {
                self.authentication_service.client_ids_for_user(user_id)
            },
            AddressEnvelope::ToAllPlayersInGame(game_id) => {
                let player_ids = self.games_service.players_in_game(game_id);
                player_ids
                    .into_iter()
                    .flat_map(|player_id| self.user_ids_for_player(game_id, player_id))
                    .flat_map(|user_id| self.authentication_service.client_ids_for_user(user_id))
                    .collect()
            },
        };

        ServerResponseWithClientIds {
            client_ids,
            response: server_response_with_address.response,
        }
    }

    fn process_network_command(
        client_id: ClientId,
        network_command: &NetworkCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        match network_command {
            NetworkCommand::Ping { id, elapsed } => {
                Ok(vec![ServerResponseWithAddress {
                    address:  AddressEnvelope::ToClient(client_id),
                    response: ServerResponse::Network(NetworkResponse::Pong {
                        id:      *id,
                        elapsed: *elapsed,
                    }),
                }])
            },
        }
    }

    fn process_internal(
        &mut self,
        client_command_with_client_id: &ClientCommandWithClientId,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        let client_id = client_command_with_client_id.client_id();
        match client_command_with_client_id.command() {
            ClientCommand::Network(network_command) => {
                Self::process_network_command(client_id, network_command)
            },
            ClientCommand::Authentication(authentication_command) => {
                self.authentication_service
                    .process_authentication_command(client_id, authentication_command)
            },
            ClientCommand::Lobby(lobby_command) => {
                let requesting_user_id = self.authentication_service.lookup_user_id(client_id)?;
                self.games_service.process_lobby_command(
                    &self.authentication_service.user_info(requesting_user_id),
                    lobby_command,
                )
            },
            ClientCommand::Game(game_id, game_command) => {
                let requesting_user_id = self.authentication_service.lookup_user_id(client_id)?;
                self.games_service
                    .process_command(*game_id, requesting_user_id, game_command)
            },
        }
    }

    #[must_use]
    pub fn process(
        &mut self,
        client_command_with_client_id: &ClientCommandWithClientId,
    ) -> Vec<ServerResponseWithClientIds> {
        let client_id = client_command_with_client_id.client_id();
        let responses = self.process_internal(client_command_with_client_id);

        let flattened = responses
            .map_err(|response| {
                vec![ServerResponseWithAddress {
                    address:  AddressEnvelope::ToClient(client_id),
                    response: *response,
                }]
            })
            .unwrap_or_else(identity);

        flattened
            .into_iter()
            .map(|response| self.translate_response(response))
            .collect()
    }
}
