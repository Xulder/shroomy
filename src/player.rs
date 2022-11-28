use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

use crate::{spritesheet::spawn_sprite, spritesheet::SpriteSheet, TILE_SIZE};

pub struct PlayerPlugin;

#[derive(Component, Inspectable)]
pub struct Player {
    pub speed: f32,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player)
            .add_system(player_movement);
    }
}

fn player_movement(
    mut player_query: Query<(&Player, &mut Transform)>,
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (player, mut transform) = player_query.single_mut();

    if keyboard.pressed(KeyCode::W) {
        transform.translation.y += player.speed * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::S) {
        transform.translation.y -= player.speed * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::A) {
        transform.translation.x -= player.speed * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::D) {
        transform.translation.x += player.speed * time.delta_seconds();
    }
}

fn spawn_player(mut commands: Commands, spritesheet: Res<SpriteSheet>) {
    let player = spawn_sprite(
        &mut commands,
        &spritesheet,
        0,
        Color::rgb(1., 1., 1.),
        Vec3::new(0., 0., 900.),
    );

    commands
        .entity(player)
        .insert(Name::new("Player"))
        .insert(Player { speed: 1.2 })
        .id();
}
