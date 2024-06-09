use std::error::Error;
use std::net::{AddrParseError, IpAddr, Ipv4Addr, SocketAddr};

use networking_renet_shared::DEFAULT_PORT;

pub mod networking;
pub mod networking_visualisation;

pub fn server_address(parameter: Option<String>) -> Result<SocketAddr, impl Error> {
    match parameter {
        None => {
            Ok::<SocketAddr, AddrParseError>(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                DEFAULT_PORT,
            ))
        },
        Some(address) => {
            let address = address.parse::<SocketAddr>()?;
            Ok(address)
        },
    }
}
