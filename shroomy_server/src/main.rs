use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

use bevy::{
    app::AppExit,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_renet::{
    renet::{RenetClient, RenetServer, ServerAuthentication, ServerConfig, ServerEvent},
    RenetServerPlugin,
};
use rand::{thread_rng, Rng};
use renet_visualizer::RenetServerVisualizer;
use shroomy_common::{
    server_connection_config, ClientChannel, NetworkedEntities, Player, PlayerInput, ServerChannel,
    ServerMessages, PROTOCOL_ID,
};

// TODO: Move to player module
const PLAYER_MOVE_SPEED: f32 = 5.0;

// TODO: Refactor for multiple instances
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
    app.add_plugin(FrameTimeDiagnosticsPlugin::default());
    app.add_plugin(LogDiagnosticsPlugin::default());
    app.add_plugin(EguiPlugin);

    app.insert_resource(ServerLobby::default());
    app.insert_resource(new_renet_server());
    app.insert_resource(RenetServerVisualizer::<200>::default());

    app.add_system(server_update_system);
    app.add_system(server_network_sync);
    // app.add_system_to_stage(CoreStage::PostUpdate, server_update_system);
    // app.add_system_to_stage(CoreStage::PostUpdate, server_network_sync);
    app.add_system(move_players_system);
    app.add_system(update_visualizer_system);

    // NOTE: This might be useful down the line for observing instances visually without having to interact with client windows
    // Any sprite/asset related things could potentially be moved to common or a new crate if this is done.
    // app.add_startup_system(admin_camera?);

    app.run();
}

#[allow(clippy::too_many_arguments)]
fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<ServerLobby>,
    mut server: ResMut<RenetServer>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    players: Query<(Entity, &Player, &Transform)>,
) {
    for event in server_events.iter() {
        match event {
            // TODO: A lot of this player creation code should be thrown into a player module
            // Obviously a huge expansion of content should be added to this code
            // Character customization, stats, abilities, associated account(s), etc.
            ServerEvent::ClientConnected(id, _) => {
                println!("Player {} connected.", id);
                visualizer.add_client(*id);

                for (entity, player, transform) in players.iter() {
                    let translation: [f32; 3] = transform.translation.into();
                    let message = bincode::serialize(&ServerMessages::PlayerCreate {
                        entity,
                        id: player.id,
                        translation,
                    })
                    .unwrap();
                    server.send_message(*id, ServerChannel::ServerMessages, message);
                }

                // let transform = Transform::from_xyz(0.0, 0.0, 0.0);
                // NOTE: Testing purposes so clients don't stack
                let mut rng = thread_rng();
                let transform = Transform::from_xyz(
                    rng.gen_range(-50.0..50.0),
                    rng.gen_range(-50.0..50.0),
                    900.0,
                );
                let player_entity = commands
                    .spawn(TransformBundle {
                        local: transform,
                        ..Default::default()
                    })
                    .insert(PlayerInput::default())
                    .insert(Player { id: *id })
                    .id();

                lobby.players.insert(*id, player_entity);

                let translation = transform.translation.into();
                let message = bincode::serialize(&ServerMessages::PlayerCreate {
                    id: *id,
                    entity: player_entity,
                    translation,
                })
                .unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
            ServerEvent::ClientDisconnected(id) => {
                println!("Player {} disconnected.", id);
                visualizer.remove_client(*id);
                if let Some(player_entity) = lobby.players.remove(id) {
                    commands.entity(player_entity).despawn();
                }

                let message =
                    bincode::serialize(&ServerMessages::PlayerRemove { id: *id }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
        }
    }

    for client_id in server.clients_id().into_iter() {
        /* TODO: Consume and broadcast player commands here
         * The structure should look similar to the below while loop, but match against PlayerCommand
         * varients and use `broadcast_message`
         */
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let input: PlayerInput = bincode::deserialize(&message).unwrap();
            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(input);
            }
        }
    }
}

fn update_visualizer_system(
    mut egui_context: ResMut<EguiContext>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    server: Res<RenetServer>,
) {
    visualizer.update(&server);
    visualizer.show_window(egui_context.ctx_mut());
}

//
#[allow(clippy::type_complexity)]
fn server_network_sync(
    mut server: ResMut<RenetServer>,
    query: Query<(Entity, &Transform), With<Player>>,
) {
    let mut networked_entities = NetworkedEntities::default();
    for (entity, transform) in query.iter() {
        networked_entities.entities.push(entity);
        networked_entities
            .translations
            .push(transform.translation.into());
    }

    let sync_message = bincode::serialize(&networked_entities).unwrap();
    server.broadcast_message(ServerChannel::NetworkedEntities, sync_message);
}

// NOTE: Uses a normalized vec for determining direction so diagnals are ezclap
fn move_players_system(mut query: Query<(&mut Transform, &PlayerInput)>) {
    for (mut transform, input) in query.iter_mut() {
        let x = (input.right as i8 - input.left as i8) as f32;
        let y = (input.up as i8 - input.down as i8) as f32;
        let direction = Vec2::new(x, y).normalize_or_zero();
        transform.translation.x = transform.translation.x + (direction.x * PLAYER_MOVE_SPEED);
        transform.translation.y = transform.translation.y + (direction.y * PLAYER_MOVE_SPEED);
    }
}
