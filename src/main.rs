use bevy::{
    prelude::*,
    render::{camera::ScalingMode, prelude::ImagePlugin},
};

mod debug;
mod player;
mod spritesheet;
mod tilemap;

use debug::DebugPlugin;
use player::PlayerPlugin;
use spritesheet::SpriteSheetPlugin;

pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const RESOLUTION: f32 = 16.0 / 9.0;
pub const TILE_SIZE: f32 = 0.3;

fn main() {
    App::new()
        .insert_resource(ClearColor(CLEAR))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: 1600.0,
                        height: 900.0,
                        title: "Shroomy".to_string(),
                        resizable: false,
                        ..default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_startup_system(spawn_camera)
        .add_plugin(PlayerPlugin)
        .add_plugin(SpriteSheetPlugin)
        .add_plugin(DebugPlugin)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();

    camera.projection.top = 1.0;
    camera.projection.bottom = -1.0;

    camera.projection.right = 1.0 * RESOLUTION;
    camera.projection.left = -1.0 * RESOLUTION;

    camera.projection.scaling_mode = ScalingMode::None;

    commands.spawn(camera);
}
