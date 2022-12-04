use std::{
    collections::HashMap,
    net::UdpSocket,
    thread::sleep,
    time::{Duration, SystemTime},
};

use bevy::{
    app::AppExit,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window,
};
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_renet::{
    renet::{ClientAuthentication, RenetClient, RenetError},
    run_if_client_connected, RenetClientPlugin,
};
use renet_visualizer::{RenetClientVisualizer, RenetVisualizerStyle};
use shroomy_common::{
    client_connection_config, ClientChannel, NetworkedEntities, PlayerCommand, PlayerInput,
    ServerChannel, ServerMessages, PROTOCOL_ID,
};

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

fn new_renet_client() -> RenetClient {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let connection_config = client_connection_config();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugin(RenetClientPlugin::default());
    app.add_plugin(FrameTimeDiagnosticsPlugin::default());
    app.add_plugin(LogDiagnosticsPlugin::default());
    app.add_plugin(EguiPlugin);

    // TODO: Implement player commands
    // app.add_event::<PlayerCommand>();

    app.insert_resource(ClientLobby::default());
    app.insert_resource(PlayerInput::default());
    app.insert_resource(new_renet_client());
    app.insert_resource(NetworkMapping::default());

    app.add_system(window::close_on_esc);
    app.add_system(player_input);
    // app.add_system(camera_follow);
    app.add_system(client_send_input.with_run_criteria(run_if_client_connected));
    // app.add_system(client_send_player_commands.with_run_criteria(run_if_client_connected));
    app.add_system_to_stage(
        CoreStage::PostUpdate,
        client_sync_players.with_run_criteria(run_if_client_connected),
    );

    app.insert_resource(RenetClientVisualizer::<200>::new(
        RenetVisualizerStyle::default(),
    ));
    app.add_system(update_visualizer_system);

    app.add_startup_system(setup_camera);
    app.add_startup_system(load_player_spritesheet);
    app.add_system(panic_on_error_system);
    app.add_system_to_stage(CoreStage::PostUpdate, on_app_exit);

    app.run();
}

pub fn on_app_exit(app_exit_events: EventReader<AppExit>, mut client: ResMut<RenetClient>) {
    if !app_exit_events.is_empty() {
        client.disconnect();
        sleep(Duration::from_secs_f32(0.1));
    }
}

/// panic on netcode error
fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    for e in renet_error.iter() {
        panic!("{}", e);
    }
}

// NOTE: Should eventually mess with this and the style.
fn update_visualizer_system(
    mut egui_context: ResMut<EguiContext>,
    mut visualizer: ResMut<RenetClientVisualizer<200>>,
    client: Res<RenetClient>,
    mut show_visualizer: Local<bool>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    visualizer.add_network_info(client.network_info());
    if keyboard_input.just_pressed(KeyCode::F1) {
        *show_visualizer = !*show_visualizer;
    }
    if *show_visualizer {
        visualizer.show_window(egui_context.ctx_mut());
    }
}

fn player_input(keyboard_input: Res<Input<KeyCode>>, mut player_input: ResMut<PlayerInput>) {
    player_input.left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    player_input.right =
        keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);
    player_input.up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
    player_input.down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
}

fn client_send_input(player_input: Res<PlayerInput>, mut client: ResMut<RenetClient>) {
    let input_message = bincode::serialize(&*player_input).unwrap();

    client.send_message(ClientChannel::Input, input_message);
}

// TODO: Impliment player commands see `shroomy_server/src/main.rs:133`
// NOTE: Producers simply have to send a PlayerCommand to an EventWriter (just add one to a system after adding the event to the app)
#[allow(unused)]
fn client_send_player_commands(
    mut player_commands: EventReader<PlayerCommand>,
    mut client: ResMut<RenetClient>,
) {
    for command in player_commands.iter() {
        let command_message = bincode::serialize(command).unwrap();
        client.send_message(ClientChannel::Command, command_message);
    }
}

fn client_sync_players(
    mut commands: Commands,
    player_spritesheet: Res<PlayerSpriteSheet>,
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    let client_id = client.client_id();
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerCreate {
                entity,
                id,
                translation,
            } => {
                println!("Player {} connected.", id);
                let mut sprite = TextureAtlasSprite::new(0);
                // offsets the color of other client's sprites
                sprite.color = if client_id == id {
                    Color::rgb(1.0, 1.0, 1.0)
                } else {
                    Color::rgb(1.0, 0.6, 0.6)
                };
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

                if client_id == id {
                    client_entity.insert(ControlledPlayer);
                }

                let player_info = PlayerInfo {
                    server_entity: entity,
                    client_entity: client_entity.id(),
                };
                lobby.players.insert(id, player_info);
                network_mapping.0.insert(entity, client_entity.id());
            }
            ServerMessages::PlayerRemove { id } => {
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
    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let networked_entities: NetworkedEntities = bincode::deserialize(&message).unwrap();

        for i in 0..networked_entities.entities.len() {
            if let Some(entity) = network_mapping.0.get(&networked_entities.entities[i]) {
                let translation = networked_entities.translations[i].into();
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

// NOTE: This is kept isolated as a system for scaling purposes.
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// TODO: Implement following camera
// NOTE: Potentially look into smooth bevy cameras [https://docs.rs/smooth-bevy-cameras/latest/smooth_bevy_cameras/]
//       I imagine you could just access the `ControlledPlayer` with it's transform too.
// fn camera_follow() {

// }
