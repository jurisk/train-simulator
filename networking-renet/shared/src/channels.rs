use std::time::Duration;

use bevy_renet::renet::{ChannelConfig, ConnectionConfig, SendType};

pub enum ClientChannel {
    Command,
}

pub enum ServerChannel {
    ServerMessages,
    NetworkedEntities,
}

impl From<ClientChannel> for u8 {
    fn from(channel_id: ClientChannel) -> Self {
        match channel_id {
            ClientChannel::Command => 0,
        }
    }
}

impl ClientChannel {
    #[must_use]
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![ChannelConfig {
            channel_id:             Self::Command.into(),
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type:              SendType::ReliableOrdered {
                resend_time: Duration::ZERO,
            },
        }]
    }
}

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::NetworkedEntities => 0,
            ServerChannel::ServerMessages => 1,
        }
    }
}

impl ServerChannel {
    #[must_use]
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id:             Self::NetworkedEntities.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type:              SendType::Unreliable,
            },
            ChannelConfig {
                channel_id:             Self::ServerMessages.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type:              SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
        ]
    }
}

#[must_use]
pub fn connection_config() -> ConnectionConfig {
    ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config:   ClientChannel::channels_config(),
        server_channels_config:   ServerChannel::channels_config(),
    }
}
