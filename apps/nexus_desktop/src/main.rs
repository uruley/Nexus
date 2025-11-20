use anchor::{AnchorPlugin, Velocity};
use anyhow::Result;
use bevy::app::App;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::core_pipeline::CorePipelinePlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::sprite::SpritePlugin;
use bevy::transform::TransformPlugin;
use bevy::window::WindowPlugin;
use bevy::winit::WinitPlugin;
use tracing::info;
use world_state::Collider;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!(target: "nexus_desktop", "Launching Nexus desktop app");

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.08)))
        .add_plugins((
            MinimalPlugins,
            WindowPlugin::default(),
            WinitPlugin::default(),
            TransformPlugin,
            RenderPlugin::default(),
            CorePipelinePlugin::default(),
            SpritePlugin::default(),
        ))
        .add_plugins(AnchorPlugin)
        .add_systems(Startup, setup_scene)
        .run();

    Ok(())
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.7, 1.0),
                custom_size: Some(Vec2::splat(120.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 200.0, 0.0),
            ..Default::default()
        },
        Velocity(Vec3::ZERO),
        Collider {
            half_extents: Vec3::new(60.0, 60.0, 0.0),
        },
    ));
}
