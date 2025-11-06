use bevy::{
    core_pipeline::{clear_color::ClearColorConfig, core_3d::Camera3dBundle},
    prelude::*,
    window::WindowPlugin,
    winit::WinitPlugin,
};

fn main() {
    tracing_subscriber::fmt().with_target(false).init();

    App::new()
        .add_plugins((
            MinimalPlugins,
            TransformPlugin,
            HierarchyPlugin,
            DiagnosticsPlugin,
            InputPlugin,
            WindowPlugin::default(),
            WinitPlugin::default(),
            bevy::render::RenderPlugin::default(),
            bevy::core_pipeline::CorePipelinePlugin::default(),
            bevy::sprite::SpritePlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.05, 0.05, 0.08)),
            ..default()
        },
        ..default()
    });

    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        ..default()
    });
}
