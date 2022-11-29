use bevy::prelude::*;
use bevy::renet::renet::{
    ChannelConfig, ReliableChannelConfig, RenetConnectionConfig, UnreliableChannelConfig,
    NETCODE_KEY_BYTES,
};

pub const PRIVATE_KEY: [&u8; NETCODE_KEY_BYTES] = b"super secret key";
pub const PROTOCOL_ID: u8 = 7;

#[derive(Debug, Component)]
pub struct Player {
    pub id: u64,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component, Resource)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

pub enum ClientChannel {
    Input,
    Command,
}

pub enum ServerChannel {
    ServerMessages,
    NetworkedEntities,
}

#[derive(Debug, Serialize, Deserialize, Component)]


// #[cfg(test)]
// mod tests {
// }
