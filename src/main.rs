use bevy::{
    prelude::*,
    render::{camera::ScalingMode, texture::ImageSettings},
};

pub mod debug;
pub mod player;

// use debug::DebugPlugin;
use player::PlayerPlugin;

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const RESOLUTION: f32 = 16.0 / 9.0;
pub const TILE_SIZE: f32 = 0.3;

fn main() {
    App::new()
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(WindowDescriptor {
            width: 1600.0,
            height: 900.0,
            title: "Shroomy".to_string(),
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_startup_system(spawn_camera)
        .add_plugin(PlayerPlugin)
        // .add_plugin(DebugPlugin)
        .add_startup_system_to_stage(StartupStage::PreStartup, load_spritesheet)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();

    camera.projection.top = 1.0;
    camera.projection.bottom = -1.0;

    camera.projection.right = 1.0 * RESOLUTION;
    camera.projection.left = -1.0 * RESOLUTION;

    camera.projection.scaling_mode = ScalingMode::None;

    commands.spawn_bundle(camera);
}

struct SpriteSheet(Handle<TextureAtlas>);

fn load_spritesheet(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = assets.load("devart/shroomy-ph.png");
    let atlas = TextureAtlas::from_grid_with_padding(
        image,
        Vec2::splat(32.0),
        1,
        1,
        Vec2::splat(2.0),
        Vec2::splat(0.0),
    );

    let atlas_handle = texture_atlases.add(atlas);

    commands.insert_resource(SpriteSheet(atlas_handle));
}
