use crate::{
    spritesheet::{spawn_sprite, SpriteSheet},
    TILE_SIZE,
};
use bevy::prelude;

pub struct TileMapPlugin;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_simple_map);
    }
}

// TODO: Pick map making software
// TODO: Make map
// TODO: Make code to read map and display it
fn create_simple_map(mut commands: Commands, spritesheet: Res<SpriteSheet>) {}
