use anchor::AnchorPlugin;
use app_core::HudPlugin;
use bevy::{
    asset::AssetPlugin,
    core_pipeline::{clear_color::ClearColorConfig, core_3d::Camera3dBundle},
    prelude::*,
    window::WindowPlugin,
    winit::WinitPlugin,
};
use http_api::HttpApiPlugin;
use tracing::Level;
use tracing_subscriber::fmt::time::UtcTime;

fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(Level::INFO)
        .with_timer(UtcTime::rfc_3339())
        .init();

    App::new()
        .add_plugins((
            MinimalPlugins,
            TransformPlugin,
            HierarchyPlugin,
            DiagnosticsPlugin,
            InputPlugin,
            AssetPlugin::default(),
            WindowPlugin::default(),
            WinitPlugin::default(),
            bevy::render::RenderPlugin::default(),
            bevy::core_pipeline::CorePipelinePlugin::default(),
            bevy::sprite::SpritePlugin::default(),
            bevy::pbr::PbrPlugin::default(),
            AnchorPlugin::default(),
            HttpApiPlugin,
            HudPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 400.0,
    });

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

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10_000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
        ..default()
    });
}
