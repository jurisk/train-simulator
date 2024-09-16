use std::collections::HashMap;

use log::{info, warn};
use shared_domain::client_command::{AccessToken, AuthenticationCommand};
use shared_domain::server_response::{
    AddressEnvelope, AuthenticationError, AuthenticationResponse, ServerError, ServerResponse,
    ServerResponseWithAddress, UserInfo,
};
use shared_domain::{ClientId, UserId, UserName};
use uuid::uuid;

use crate::connection_registry::ConnectionRegistry;

pub(crate) struct AuthenticationService {
    connection_registry: ConnectionRegistry,
    user_infos:          HashMap<UserId, UserInfo>,
}

impl AuthenticationService {
    pub(crate) fn new() -> Self {
        Self {
            connection_registry: ConnectionRegistry::new(),
            // Later: Have a proper user database.
            user_infos:          vec![
                (
                    UserId::new(uuid!("ee6b4aa1-67e0-4d6b-a42c-56320f61390e")),
                    UserName::new("Juris".to_string()),
                ),
                (
                    UserId::new(uuid!("dd761bc8-cc22-4035-aab9-c79ab4a3b941")),
                    UserName::new("Isaak".to_string()),
                ),
                (
                    UserId::new(uuid!("2628b18e-cd05-4be3-a6ad-05b9128ab01f")),
                    UserName::new("Jānis".to_string()),
                ),
                (
                    UserId::new(uuid!("e4eca11c-f88b-4b45-8046-ae93b99fa9df")),
                    UserName::new("Арцём".to_string()),
                ),
                (
                    UserId::new(uuid!("c11f557b-57d8-4820-a363-615fe024155d")),
                    UserName::new("Imants".to_string()),
                ),
            ]
            .into_iter()
            .map(|(id, name)| (id, UserInfo { id, name }))
            .collect(),
        }
    }

    pub(crate) fn user_info(&self, user_id: UserId) -> UserInfo {
        match self.user_infos.get(&user_id) {
            None => {
                info!("Failed to find user_name for {user_id:?}, returning a random name.");
                // Later: This is just for debug, we should stop doing this
                UserInfo {
                    id:   user_id,
                    name: UserName::random(user_id.hash_to_u64()),
                }
            },
            Some(user_info) => user_info.clone(),
        }
    }

    pub(crate) fn client_ids_for_user(&self, user_id: UserId) -> Vec<ClientId> {
        match self.connection_registry.get_client_id(&user_id) {
            None => {
                warn!("Failed to find client_id for {user_id:?}");
                vec![]
            },
            Some(client_id) => vec![*client_id],
        }
    }

    pub(crate) fn process_authentication_command(
        &mut self,
        client_id: ClientId,
        authentication_command: &AuthenticationCommand,
    ) -> Result<Vec<ServerResponseWithAddress>, Box<ServerResponse>> {
        match authentication_command {
            AuthenticationCommand::Login(user_id, access_token) => {
                // Later: Check the token for validity.
                if *access_token == AccessToken::new("valid-token".to_string()) {
                    self.connection_registry.register(*user_id, client_id);

                    Ok(vec![ServerResponseWithAddress::new(
                        AddressEnvelope::ToClient(client_id),
                        ServerResponse::Authentication(AuthenticationResponse::LoginSucceeded(
                            *user_id,
                        )),
                    )])
                } else {
                    Err(Box::new(ServerResponse::Authentication(
                        AuthenticationResponse::Error(AuthenticationError::LoginFailed),
                    )))
                }
            },
            AuthenticationCommand::Logout => {
                self.connection_registry.unregister_by_client_id(&client_id);

                Ok(vec![ServerResponseWithAddress::new(
                    AddressEnvelope::ToClient(client_id),
                    ServerResponse::Authentication(AuthenticationResponse::LogoutSucceeded),
                )])
            },
        }
    }

    pub(crate) fn lookup_user_id(
        &self,
        client_id: ClientId,
    ) -> Result<UserId, Box<ServerResponse>> {
        match self.connection_registry.get_user_id(&client_id) {
            None => Err(Box::new(ServerResponse::Error(ServerError::NotAuthorized))),
            Some(requesting_user_id) => Ok(*requesting_user_id),
        }
    }
}
