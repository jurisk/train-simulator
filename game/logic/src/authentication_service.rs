use std::collections::HashMap;

use log::{info, warn};
use shared_domain::client_command::AuthenticationCommand;
use shared_domain::server_response::{
    AddressEnvelope, AuthenticationResponse, ServerError, ServerResponse, ServerResponseWithAddress,
};
use shared_domain::{ClientId, PlayerId, PlayerName};
use uuid::uuid;

use crate::connection_registry::ConnectionRegistry;

pub(crate) struct AuthenticationService {
    connection_registry: ConnectionRegistry,
    player_names:        HashMap<PlayerId, PlayerName>,
}

impl AuthenticationService {
    pub(crate) fn new() -> Self {
        Self {
            connection_registry: ConnectionRegistry::new(),
            // Later: Have a proper player database.
            player_names:        HashMap::from([
                (
                    PlayerId(uuid!("ee6b4aa1-67e0-4d6b-a42c-56320f61390e")),
                    PlayerName("Juris".to_string()),
                ),
                (
                    PlayerId(uuid!("dd761bc8-cc22-4035-aab9-c79ab4a3b941")),
                    PlayerName("Isaak".to_string()),
                ),
                (
                    PlayerId(uuid!("2628b18e-cd05-4be3-a6ad-05b9128ab01f")),
                    PlayerName("Jānis".to_string()),
                ),
                (
                    PlayerId(uuid!("e4eca11c-f88b-4b45-8046-ae93b99fa9df")),
                    PlayerName("Арцём".to_string()),
                ),
            ]),
        }
    }

    pub(crate) fn player_name(&self, player_id: PlayerId) -> PlayerName {
        match self.player_names.get(&player_id) {
            None => {
                info!("Failed to find player_name for {player_id:?}, returning a random name.");
                PlayerName::random(player_id.hash_to_u64())
            },
            Some(player_name) => player_name.clone(),
        }
    }

    pub(crate) fn client_ids_for_player(&self, player_id: PlayerId) -> Vec<ClientId> {
        match self.connection_registry.get_client_id(&player_id) {
            None => {
                warn!("Failed to find client_id for {player_id:?}");
                vec![]
            },
            Some(client_id) => vec![*client_id],
        }
    }

    pub(crate) fn process_authentication_command(
        &mut self,
        client_id: ClientId,
        authentication_command: AuthenticationCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
        match authentication_command {
            AuthenticationCommand::Login(player_id, access_token) => {
                // Later: Check the token for validity.
                if access_token.0 == "valid-token" {
                    self.connection_registry.register(player_id, client_id);

                    Ok(vec![ServerResponseWithAddress::new(
                        AddressEnvelope::ToClient(client_id),
                        ServerResponse::Authentication(AuthenticationResponse::LoginSucceeded(
                            player_id,
                        )),
                    )])
                } else {
                    Err(ServerResponse::Authentication(
                        AuthenticationResponse::LoginFailed,
                    ))
                }
            },
            AuthenticationCommand::Logout => {
                self.connection_registry.unregister_by_client_id(client_id);

                Ok(vec![ServerResponseWithAddress::new(
                    AddressEnvelope::ToClient(client_id),
                    ServerResponse::Authentication(AuthenticationResponse::LogoutSucceeded),
                )])
            },
        }
    }

    pub(crate) fn lookup_player_id(&self, client_id: ClientId) -> Result<PlayerId, ServerResponse> {
        match self.connection_registry.get_player_id(&client_id) {
            None => Err(ServerResponse::Error(ServerError::NotAuthorized)),
            Some(requesting_player_id) => Ok(*requesting_player_id),
        }
    }
}
