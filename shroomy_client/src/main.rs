use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode, Client, ConnectionConfiguration, ConnectionEvent,
        QuinnetClientPlugin,
    },
    shared::ClientId,
};
use bevy_egui::{EguiContext, EguiPlugin};
use shroomy_common::{ClientMessage, NetworkedEntities, PlayerCommand, PlayerInput, ServerChannel, ServerMessage};

// TODO: Potentially refactor to something better optimize for modest
// multiplayer eventually (~100 players per in game area/region instance)
#[derive(Default, Resource)]
struct NetworkMapping(HashMap<Entity, Entity>);

// TODO: Player related components and DTOs should be modularized
#[derive(Component)]
struct ControlledPlayer;

#[derive(Debug)]
struct PlayerInfo {
    client_entity: Entity,
    server_entity: Entity,
}

#[derive(Debug, Default, Resource)]
struct ClientLobby {
    players: HashMap<u64, PlayerInfo>,
}

#[derive(Debug, Resource)]
struct PlayerSpriteSheet(Handle<TextureAtlas>);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugin(QuinnetClientPlugin::default());
    //app.add_plugin(FrameTimeDiagnosticsPlugin::default());
    //app.add_plugin(LogDiagnosticsPlugin::default());
    app.add_plugin(EguiPlugin);

    // TODO: Implement player commands
    // app.add_event::<PlayerCommand>();

    app.insert_resource(ClientLobby::default());
    app.insert_resource(PlayerInput::default());
    app.insert_resource(NetworkMapping::default());

    app.add_system(player_input);
    // app.add_system(camera_follow);
    app.add_system(client_send_input);
    // app.add_system(client_send_player_commands.with_run_criteria(run_if_client_connected));
    app.add_system(client_sync_players);
    app.add_system(handle_client_events);

    app.add_startup_system(start_connection);
    app.add_startup_system(setup_camera);
    app.add_startup_system(load_player_spritesheet);

    app.run();
}

fn start_connection(mut client: ResMut<Client>) {
    client.open_connection(
        ConnectionConfiguration::new("127.0.0.1".to_string(), 6000, "0.0.0.0".to_string(), 0),
        CertificateVerificationMode::SkipVerification,
    );
}

fn player_input(keyboard_input: Res<Input<KeyCode>>, mut player_input: ResMut<PlayerInput>) {
    player_input.left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    player_input.right =
        keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);
    player_input.up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
    player_input.down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
}

fn client_send_input(player_input: Res<PlayerInput>, mut client: ResMut<Client>) {
   // let input_message = bincode::serialize(&*player_input).unwrap();

    client
        .connection()
        .send_message(*player_input)
        .unwrap();
}

// TODO: Implement player commands see `shroomy_server/src/main.rs:133`
// NOTE: Producers simply have to send a PlayerCommand to an EventWriter (just add one to a system after adding the event to the app)
#[allow(unused)]
fn client_send_player_commands(
    mut player_commands: EventReader<PlayerCommand>,
    mut client: ResMut<Client>,
) {
    for command in player_commands.iter() {
        let command_message = bincode::serialize(command).unwrap();

        client
            .connection()
            .send_message(command_message);
    }
}

fn client_sync_players(
    mut commands: Commands,
    player_spritesheet: Res<PlayerSpriteSheet>,
    mut client: ResMut<Client>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    while let Some(message) = client
        .connection_mut()
        .try_receive_message::<ServerMessage>()
    {
        //let server_message = bincode::deserialize(&message).unwrap();
        match message {
            ServerMessage::ClientDisconnected { client_id } => {
                if let Some(_) = lobby.players.remove(&client_id) {
                    println!("{} left", client_id);
                } else {
                    warn!("ClientDisconnected for an unknown client_id: {}", client_id)
                }
            }
            ServerMessage::PlayerCreate {
                entity,
                id,
                translation,
            } => {
                println!("Player {} connected.", id);
                let mut sprite = TextureAtlasSprite::new(0);
                // offsets the color of other client's sprites
                sprite.color = Color::rgb(1.0, 1.0, 1.0);
                /*if client_id == id {
                    Color::rgb(1.0, 1.0, 1.0)
                } else {
                    Color::rgb(1.0, 0.6, 0.6)
                };*/
                sprite.custom_size = Some(Vec2::splat(64.0));

                let mut client_entity = commands.spawn(SpriteSheetBundle {
                    sprite,
                    texture_atlas: player_spritesheet.0.clone(),
                    transform: Transform {
                        translation: Vec3::from(translation),
                        ..Default::default()
                    },
                    ..Default::default()
                });

                /*if client_id == id {
                    client_entity.insert(ControlledPlayer);
                }*/

                client_entity.insert(ControlledPlayer);

                let player_info = PlayerInfo {
                    server_entity: entity,
                    client_entity: client_entity.id(),
                };
                lobby.players.insert(id, player_info);
                network_mapping.0.insert(entity, client_entity.id());
            }
            ServerMessage::PlayerRemove { id } => {
                println!("Player {} disconnected.", id);
                if let Some(PlayerInfo {
                    server_entity,
                    client_entity,
                }) = lobby.players.remove(&id)
                {
                    commands.entity(client_entity).despawn();
                    network_mapping.0.remove(&server_entity);
                }
            } // TODO: Other kinds of server messages will need to be implemented.
              // This can be abstracted down into modules onces a clear seperation of domain occurs.
              // Planning and mapping out seems like a good idea here. A lot of content will revolve
              // around messages received from the server and vise versa.
              // Enemy attacks (pve or pvp), spells, dialogue triggers, popup windows, etc.
        }
    }

    // NOTE: This is simply updating the in-memory data for entities from the server.
    // I'm not sure what the limit to the HashMap would be, so profiling tests might be necessary.
    while let Some(message) = client.connection_mut().try_receive_message::<NetworkedEntities>() {
        //let networked_entities: NetworkedEntities = bincode::deserialize(&message).unwrap();

        for i in 0..message.entities.len() {
            if let Some(entity) = network_mapping.0.get(&message.entities[i]) {
                let translation = message.translations[i].into();
                let transform = Transform {
                    translation,
                    ..Default::default()
                };
                commands.entity(*entity).insert(transform);
            }
        }
    }
}

// TODO: Should be moved to a player module
// TODO: Add animation and spritesheets to go with it
// TODO: Should set this up to load any part of an unequipped player character
/// Adds player spritesheet as a resource. This should be compatible with animation.
fn load_player_spritesheet(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("player_sprite.png");
    // NOTE: Padding should be added when full spritesheets are made.
    let atlas = TextureAtlas::from_grid(image, Vec2::splat(32.0), 1, 1, None, None);

    let atlas_handle = texture_atlases.add(atlas);

    commands.insert_resource(PlayerSpriteSheet(atlas_handle));
}

fn handle_client_events(connection_events: EventReader<ConnectionEvent>, client: ResMut<Client>) {
    if !connection_events.is_empty() {
        client
            .connection()
            .send_message(ClientMessage::Join { })
            .unwrap();

        connection_events.clear();
    }
}

// NOTE: This is kept isolated as a system for scaling purposes.
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// TODO: Implement following camera
// NOTE: Potentially look into smooth bevy cameras [https://docs.rs/smooth-bevy-cameras/latest/smooth_bevy_cameras/]
//       I imagine you could just access the `ControlledPlayer` with it's transform too.
// fn camera_follow() {

// }
