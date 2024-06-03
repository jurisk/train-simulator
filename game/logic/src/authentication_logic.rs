use shared_domain::client_command::AuthenticationCommand;
use shared_domain::server_response::{
    AddressEnvelope, AuthenticationResponse, ServerResponse, ServerResponseWithAddress,
};
use shared_domain::ClientId;

use crate::connection_registry::ConnectionRegistry;

pub fn process_authentication_command(
    connection_registry: &mut ConnectionRegistry,
    client_id: ClientId,
    authentication_command: AuthenticationCommand,
) -> Vec<ServerResponseWithAddress> {
    match authentication_command {
        AuthenticationCommand::Login(player_id, access_token) => {
            if access_token.0 == "valid-token" {
                connection_registry.register(player_id, client_id);

                vec![ServerResponseWithAddress::new(
                    AddressEnvelope::ToClient(client_id),
                    ServerResponse::Authentication(AuthenticationResponse::LoginSucceeded(
                        player_id,
                    )),
                )]
            } else {
                vec![ServerResponseWithAddress::new(
                    AddressEnvelope::ToClient(client_id),
                    ServerResponse::Authentication(AuthenticationResponse::LoginFailed),
                )]
            }
        },
        AuthenticationCommand::Logout => {
            connection_registry.unregister_by_client_id(client_id);

            vec![ServerResponseWithAddress::new(
                AddressEnvelope::ToClient(client_id),
                ServerResponse::Authentication(AuthenticationResponse::LogoutSucceeded),
            )]
        },
    }
}
