#![allow(clippy::missing_errors_doc)]

use shared_domain::client_command::AuthenticationCommand;
use shared_domain::server_response::{
    AddressEnvelope, AuthenticationResponse, ServerError, ServerResponse, ServerResponseWithAddress,
};
use shared_domain::{ClientId, PlayerId};

use crate::connection_registry::ConnectionRegistry;

pub fn process_authentication_command(
    connection_registry: &mut ConnectionRegistry,
    client_id: ClientId,
    authentication_command: AuthenticationCommand,
) -> Result<Vec<ServerResponseWithAddress>, ServerResponse> {
    match authentication_command {
        AuthenticationCommand::Login(player_id, access_token) => {
            if access_token.0 == "valid-token" {
                connection_registry.register(player_id, client_id);

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
            connection_registry.unregister_by_client_id(client_id);

            Ok(vec![ServerResponseWithAddress::new(
                AddressEnvelope::ToClient(client_id),
                ServerResponse::Authentication(AuthenticationResponse::LogoutSucceeded),
            )])
        },
    }
}

pub fn lookup_player_id(
    connection_registry: &ConnectionRegistry,
    client_id: ClientId,
) -> Result<PlayerId, ServerResponse> {
    match connection_registry.get_player_id(&client_id) {
        None => Err(ServerResponse::Error(ServerError::NotAuthorized)),
        Some(requesting_player_id) => Ok(*requesting_player_id),
    }
}
