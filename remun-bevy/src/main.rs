use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(big_hello)
        .add_startup_system(debug_overlay_setup)
        .run();
}

// Simple system
fn big_hello() {
    println!("Big Hello! 👋");
}

#[derive(Component)]
pub struct MainCamera;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default()).insert(MainCamera);
    commands.spawn(SpriteBundle {
        texture: asset_server.load("icon.png"),
        ..default()
    });
}

// UI
fn debug_overlay_setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(UiCameraConfig::default());
    //commands.spawn(ButtonBundle {
    //    background_color: 3,
    //});
}

