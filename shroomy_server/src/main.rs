use bevy::prelude::*;
use bevy_renet::{
    renet::{RenetServer, ServerAuthentication, ServerConfig, ServerEvent},
    RenetServerPlugin,
};
use shroomy_common::{
    server_connection_config, NetworkedEntities, Player, PlayerInput, ServerChannel,
    ServerMessages, PROTOCOL_ID,
};
use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>,
}

fn new_renet_server() -> RenetServer {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let connection_config = server_connection_config();
    let server_config =
        ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(RenetServerPlugin::default());
    app.insert_resource(ServerLobby::default());
    app.insert_resource(new_renet_server());

    app.add_system(server_update_system);
    app.add_system(server_network_sync);

    app.run();
}

#[allow(clippy::too_many_arguments)]
fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<ServerLobby>,
    mut server: ResMut<RenetServer>,
    players: Query<(Entity, &Player)>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("Player {} connected.", id);

                for (entity, player) in players.iter() {
                    let message = bincode::serialize(&ServerMessages::PlayerCreate {
                        entity,
                        id: player.id,
                    })
                    .unwrap();
                    server.send_message(*id, ServerChannel::ServerMessages, message);
                }

                let player_entity = commands
                    .spawn_empty()
                    .insert(PlayerInput::default())
                    .insert(Player { id: *id })
                    .id();

                lobby.players.insert(*id, player_entity);

                let message = bincode::serialize(&ServerMessages::PlayerCreate {
                    id: *id,
                    entity: player_entity,
                })
                .unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
            ServerEvent::ClientDisconnected(id) => {
                println!("Player {} disconnected.", id);
                if let Some(player_entity) = lobby.players.remove(id) {
                    commands.entity(player_entity).despawn();
                }

                let message =
                    bincode::serialize(&ServerMessages::PlayerRemove { id: *id }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn server_network_sync(mut server: ResMut<RenetServer>, query: Query<Entity, With<Player>>) {
    let mut networked_entities = NetworkedEntities::default();
    for entity in query.iter() {
        networked_entities.entities.push(entity);
    }

    let sync_message = bincode::serialize(&networked_entities).unwrap();
    server.broadcast_message(ServerChannel::NetworkedEntities, sync_message);
}
