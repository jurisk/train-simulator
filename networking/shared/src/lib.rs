use std::error::Error;
use std::net::{AddrParseError, IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::Resource;
use game_logic::server_state::ServerState;

pub mod channels;

#[derive(Resource)]
pub struct ServerStateResource(pub ServerState);

pub const DEFAULT_PORT: u16 = 5000;

pub fn parse_address(parameter: Option<String>) -> Result<SocketAddr, impl Error> {
    match parameter {
        None => {
            Ok::<SocketAddr, AddrParseError>(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                DEFAULT_PORT,
            ))
        },
        Some(address) => {
            let address = address.parse::<SocketAddr>()?;
            Ok(address)
        },
    }
}
