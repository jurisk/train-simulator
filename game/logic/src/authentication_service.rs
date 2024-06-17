use log::warn;
use shared_domain::client_command::AuthenticationCommand;
use shared_domain::server_response::{
    AddressEnvelope, AuthenticationResponse, ServerError, ServerResponse, ServerResponseWithAddress,
};
use shared_domain::{ClientId, PlayerId};

use crate::connection_registry::ConnectionRegistry;

pub(crate) struct AuthenticationService {
    connection_registry: ConnectionRegistry,
}

// TODO: This one should only return `Res<AuthenticationResponse, AuthenticationError>` to make everything simpler
impl AuthenticationService {
    pub(crate) fn new() -> Self {
        Self {
            connection_registry: ConnectionRegistry::new(),
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
