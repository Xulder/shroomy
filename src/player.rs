use bevy::prelude::*;

use crate::{SpriteSheet, TILE_SIZE};

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player {
    speed: f32,
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
    let mut sprite = TextureAtlasSprite::new(0);
    sprite.color = Color::rgb(1.0, 1.0, 1.0);
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE));

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite,
            texture_atlas: spritesheet.0.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 900.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("Player"))
        .insert(Player { speed: 1.0 })
        .id();
}
