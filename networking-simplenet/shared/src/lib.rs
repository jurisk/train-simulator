use bevy_simplenet::ChannelPack;
use serde::{Deserialize, Serialize};

// TODO: Rename away from `Test`

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestConnectMsg(pub String); // TODO: Remove

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestServerMsg(pub Vec<u8>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestServerResponse(pub u64); // TODO: Remove

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestClientMsg(pub Vec<u8>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestClientRequest(pub u64); // TODO: Remove

#[derive(Debug, Clone)]
pub struct TestChannel;
impl ChannelPack for TestChannel {
    type ClientMsg = TestClientMsg;
    type ClientRequest = TestClientRequest;
    type ConnectMsg = TestConnectMsg;
    type ServerMsg = TestServerMsg;
    type ServerResponse = TestServerResponse;
}

pub const DEFAULT_PORT: u16 = 5000;
