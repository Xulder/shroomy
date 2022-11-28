use bevy::prelude::*;

use crate::{player::Player, TILE_SIZE};

#[derive(Resource)]
pub struct SpriteSheet(pub Handle<TextureAtlas>);

pub struct SpriteSheetPlugin;

impl Plugin for SpriteSheetPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_spritesheet);
    }
}

pub fn spawn_sprite(
    command: &mut Commands,
    spritesheet: &SpriteSheet,
    index: usize,
    color: Color,
    translation: Vec3,
) -> Entity {
    let mut sprite = TextureAtlasSprite::new(index);
    sprite.color = color;
    sprite.custom_size = Some(Vec2::splat(TILE_SIZE));

    command
        .spawn(SpriteSheetBundle {
            sprite,
            texture_atlas: spritesheet.0.clone(),
            transform: Transform {
                translation,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("Player"))
        .insert(Player { speed: 1.2 })
        .id()
}

fn load_spritesheet(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("devart/shroomy-ph.png");
    let atlas = TextureAtlas::from_grid(
        image,
        Vec2::splat(32.0),
        1,
        1,
        Some(Vec2::splat(2.0)),
        Some(Vec2::splat(0.0)),
    );

    let atlas_handle = texture_atlases.add(atlas);

    commands.insert_resource(SpriteSheet(atlas_handle));
}
