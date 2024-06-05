#![allow(
    clippy::match_same_arms,
    clippy::missing_errors_doc,
    clippy::unnecessary_wraps
)]

use std::convert::identity;

use log::warn;
use shared_domain::client_command::{ClientCommand, ClientCommandWithClientId};
use shared_domain::server_response::{
    AddressEnvelope, ServerResponse, ServerResponseWithAddress, ServerResponseWithClientIds,
};
use shared_domain::{ClientId, PlayerId};

use crate::authentication_logic::{lookup_player_id, process_authentication_command};
use crate::connection_registry::ConnectionRegistry;
use crate::games::Games;

pub struct ServerState {
    pub connection_registry: ConnectionRegistry,
    games:                   Games,
}

impl ServerState {
    #[must_use]
    #[allow(clippy::missing_panics_doc, clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            connection_registry: ConnectionRegistry::new(),
            games:               Games::new(),
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

    #[allow(clippy::single_match_else)]
    fn translate_response(
        &self,
        server_response_with_address: ServerResponseWithAddress,
    ) -> ServerResponseWithClientIds {
        let client_ids = match server_response_with_address.address {
            AddressEnvelope::ToClient(client_id) => vec![client_id],
            AddressEnvelope::ToPlayer(player_id) => self.client_ids_for_player(player_id),
            AddressEnvelope::ToAllPlayersInGame(game_id) => {
                let player_ids = self.games.players_in_game(game_id);
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
                self.games
                    .process_lobby_command(requesting_player_id, lobby_command)
            },
            ClientCommand::Game(game_id, game_command) => {
                let requesting_player_id = lookup_player_id(&self.connection_registry, client_id)?;
                let game_state = self.games.lookup_game_state_mut(game_id)?;
                game_state.process_game_command(game_id, requesting_player_id, game_command)
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
