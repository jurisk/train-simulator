#![allow(
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::unnecessary_wraps
)]

use std::convert::identity;

use shared_domain::client_command::{ClientCommand, ClientCommandWithClientId, NetworkCommand};
use shared_domain::game_state::GameTime;
use shared_domain::server_response::{
    AddressEnvelope, NetworkResponse, ServerResponse, ServerResponseWithAddress,
    ServerResponseWithClientIds,
};
use shared_domain::ClientId;

use crate::authentication_service::AuthenticationService;
use crate::games_service::GamesService;

pub struct ServerState {
    authentication_service: AuthenticationService,
    games_service:          GamesService,
}

impl ServerState {
    #[must_use]
    #[allow(clippy::missing_panics_doc, clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            authentication_service: AuthenticationService::new(),
            games_service:          GamesService::new(),
        }
    }

    pub fn advance_times(&mut self, time: GameTime) {
        self.games_service.advance_times(time);
    }

    #[must_use]
    pub fn sync_games(&self) -> Vec<ServerResponseWithClientIds> {
        self.games_service
            .sync_games()
            .into_iter()
            .map(|response| self.translate_response(response))
            .collect()
    }

    #[allow(clippy::single_match_else)]
    fn translate_response(
        &self,
        server_response_with_address: ServerResponseWithAddress,
    ) -> ServerResponseWithClientIds {
        let client_ids = match server_response_with_address.address {
            AddressEnvelope::ToClient(client_id) => vec![client_id],
            AddressEnvelope::ToPlayer(player_id) => {
                self.authentication_service.client_ids_for_player(player_id)
            },
            AddressEnvelope::ToAllPlayersInGame(game_id) => {
                let player_ids = self.games_service.players_in_game(game_id);
                player_ids
                    .into_iter()
                    .flat_map(|player_id| {
                        self.authentication_service.client_ids_for_player(player_id)
                    })
                    .collect()
            },
        };

        ServerResponseWithClientIds {
            client_ids,
            response: server_response_with_address.response,
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn process_network_command(
        client_id: ClientId,
        network_command: NetworkCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        match network_command {
            NetworkCommand::Ping { id, elapsed } => {
                Ok(vec![ServerResponseWithAddress {
                    address:  AddressEnvelope::ToClient(client_id),
                    response: ServerResponse::Network(NetworkResponse::Pong { id, elapsed }),
                }])
            },
        }
    }

    fn process_internal(
        &mut self,
        client_command_with_client_id: ClientCommandWithClientId,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        let client_id = client_command_with_client_id.client_id;
        match client_command_with_client_id.command {
            ClientCommand::Network(network_command) => {
                Self::process_network_command(client_id, network_command)
            },
            ClientCommand::Authentication(authentication_command) => {
                self.authentication_service
                    .process_authentication_command(client_id, authentication_command)
            },
            ClientCommand::Lobby(lobby_command) => {
                let requesting_player_id =
                    self.authentication_service.lookup_player_id(client_id)?;
                self.games_service.process_lobby_command(
                    &self.authentication_service,
                    requesting_player_id,
                    lobby_command,
                )
            },
            ClientCommand::Game(game_id, game_command) => {
                let requesting_player_id =
                    self.authentication_service.lookup_player_id(client_id)?;
                self.games_service
                    .process_command(game_id, requesting_player_id, game_command)
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
