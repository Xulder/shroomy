use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_quinnet::{
    server::{
        certificate::CertificateRetrievalMode, ConnectionLostEvent, Endpoint, QuinnetServerPlugin,
        Server, ServerConfigurationData,
    },
    shared::ClientId,
};
use rand::{thread_rng, Rng};
use shroomy_common::{ClientMessage, NetworkedEntities, Player, PlayerInput, ServerMessage};

// TODO: Move to player module
const PLAYER_MOVE_SPEED: f32 = 5.0;

// TODO: Refactor for multiple instances
#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>,
}

fn start_listening(mut server: ResMut<Server>) {
    server
        .start_endpoint(
            ServerConfigurationData::new("127.0.0.1".to_string(), 6000, "0.0.0.0".to_string()),
            CertificateRetrievalMode::GenerateSelfSigned,
        )
        .unwrap();
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    app.add_plugin(QuinnetServerPlugin::default());
    //app.add_plugin(FrameTimeDiagnosticsPlugin::default());
    //app.add_plugin(LogDiagnosticsPlugin::default());
    app.add_plugin(EguiPlugin);

    app.insert_resource(ServerLobby::default());

    app .add_startup_system(start_listening);

    app.add_system(server_update_system);
    app.add_system(server_network_sync);
    app.add_system(move_players_system);
    app.add_system(handle_server_events);

    // NOTE: This might be useful down the line for observing instances visually without having to interact with client windows
    // Any sprite/asset related things could potentially be moved to common or a new crate if this is done.
    // app.add_startup_system(admin_camera?);

    app.run();
}

#[allow(clippy::too_many_arguments)]
fn server_update_system(
    mut commands: Commands,
    mut lobby: ResMut<ServerLobby>,
    mut server: ResMut<Server>,
    players: Query<(Entity, &Player, &Transform)>,
) {
    let endpoint = server.endpoint_mut();
    while let Ok(Some((message, client_id))) = endpoint.receive_message::<ClientMessage>() {
        match message {
            // TODO: A lot of this player creation code should be thrown into a player module
            // Obviously a huge expansion of content should be added to this code
            // Character customization, stats, abilities, associated account(s), etc.
            ClientMessage::Join {} => {
                println!("Player {} connected.", &client_id);

                for (entity, player, transform) in players.iter() {
                    let translation: [f32; 3] = transform.translation.into();
                    /*let message = bincode::serialize(&ServerMessage::PlayerCreate {
                        entity,
                        id: player.id,
                        translation,
                    })
                    .unwrap();*/
                    endpoint.send_message(client_id, ServerMessage::PlayerCreate {
                        entity,
                        id: player.id,
                        translation,
                    }).unwrap();
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
                    .insert(Player { id: client_id })
                    .id();

                lobby.players.insert(client_id, player_entity);

                let translation = transform.translation.into();

                endpoint.broadcast_message(ServerMessage::PlayerCreate {
                    id: client_id,
                    entity: player_entity,
                    translation,
                }).unwrap();
            }
            ClientMessage::Disconnect {} => {
                println!("Player {} disconnected.", &client_id);

                if let Some(player_entity) = lobby.players.remove(&client_id) {
                    commands.entity(player_entity).despawn();
                }

                //let message = bincode::serialize(&ServerMessage::PlayerRemove { id: *id }).unwrap();
                endpoint.broadcast_message(ServerMessage::PlayerRemove { id: client_id }).unwrap();
            }
        }
    }

        /* TODO: Consume and broadcast player commands here
         * The structure should look similar to the below while loop, but match against PlayerCommand
         * varients and use `broadcast_message`
         */
        while let Ok(Some((message, client_id))) = endpoint.receive_message::<PlayerInput>() {
            //let input: PlayerInput = bincode::deserialize(&message).unwrap();
            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(message);
            }
        }
}

fn handle_server_events(
    mut connection_lost_events: EventReader<ConnectionLostEvent>,
    mut server: ResMut<Server>,
    mut users: ResMut<ServerLobby>,
) {
    // The server signals us about users that lost connection
    for client in connection_lost_events.iter() {
        handle_disconnect(server.endpoint_mut(), &mut users, client.id);
    }
}

/// Shared disconnection behaviour, whether the client lost connection or asked to disconnect
fn handle_disconnect(endpoint: &mut Endpoint, users: &mut ResMut<ServerLobby>, client_id: ClientId) {
    // Remove this user
    if let Some(player) = users.players.remove(&client_id) {
        // Broadcast its deconnection

        endpoint
            .send_group_message(
                users.players.keys().into_iter(),
                ServerMessage::ClientDisconnected {
                    client_id,
                },
            )
            .unwrap();
        info!("{} disconnected", client_id);
    } else {
        warn!(
            "Received a Disconnect from an unknown or disconnected client: {}",
            client_id
        )
    }
}

//
#[allow(clippy::type_complexity)]
fn server_network_sync(
    server: Res<Server>,
    query: Query<(Entity, &Transform), With<Player>>,
) {
    let mut networked_entities = NetworkedEntities::default();
    for (entity, transform) in query.iter() {
        networked_entities.entities.push(entity);
        networked_entities
            .translations
            .push(transform.translation.into());
    }

    //let sync_message = bincode::serialize(&networked_entities).unwrap();
    server.endpoint().broadcast_message(&networked_entities).unwrap();
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
