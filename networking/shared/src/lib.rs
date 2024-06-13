use bevy_simplenet::ChannelPack;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EncodedServerMsg(pub Vec<u8>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EncodedClientMsg(pub Vec<u8>);

#[derive(Debug, Clone)]
pub struct GameChannel;
impl ChannelPack for GameChannel {
    type ClientMsg = EncodedClientMsg;
    type ClientRequest = ();
    type ConnectMsg = ();
    type ServerMsg = EncodedServerMsg;
    type ServerResponse = ();
}

pub const PORT: u16 = 8080;
