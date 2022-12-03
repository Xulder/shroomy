use bevy::prelude::*;

use serde::{Deserialize, Serialize};
use bevy_quinnet::shared::ClientId;

// TODO: Player related components should be brought into a player module.
#[derive(Debug, Component)]
pub struct Player {
    pub id: u64,
}

// NOTE: Gamepads are supported in bevy https://bevy-cheatbook.github.io/input/gamepad.html
// Honestly might just keep this as is. It's simple and easy to integrate additional movement rules on top.
// Serves it's purpose well for handling binary input.
// Only consideration would implementing controller support, which could easily be worked into another struct.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component, Resource)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum PlayerCommand {
    BasicAttack { cast_at: Vec2 },
}

// NOTE: I'm not really sure what more would be added either set of channels.
// I suppose when there's a reasonable divide of concerns under either `Input` or `Command`
//  they could be split down into more niche variants.
/*pub enum ClientChannel {
    Input,
    Command,
}*/

pub enum ServerChannel {
    ServerMessages,
    NetworkedEntities,
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessage {
    PlayerCreate {
        entity: Entity,
        id: u64,
        translation: [f32; 3],
    },
    PlayerRemove {
        id: u64,
    },
    ClientDisconnected {
        client_id: ClientId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Join { },
    Disconnect {},
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetworkedEntities {
    pub entities: Vec<Entity>,
    pub translations: Vec<[f32; 3]>,
}

/*impl From<ClientChannel> for u8 {
    fn from(channel_id: ClientChannel) -> Self {
        match channel_id {
            ClientChannel::Command => 0,
            ClientChannel::Input => 1,
        }
    }
}*/

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::NetworkedEntities => 0,
            ServerChannel::ServerMessages => 1,
        }
    }
}

// TODO: Add reasonable tests to all of this.
// #[cfg(test)]
// mod tests {
// }
