use bevy::prelude::*;
use bevy_renet::{
    renet::{RenetServer, ServerAuthentication, ServerConfig, ServerEvent},
    RenetServerPlugin,
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
    let server_config = ServerConfig::new(64, PROTOCOL_ID)
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(RenetServerPlugin::default());
    app.insert_resource(ServerLobby::default());
    app.insert_resource(new_renet_server());
}
