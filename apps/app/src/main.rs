use anchor::{AnchorPlugin, SimulationMode};
use app_core::{HudPlugin, PerceptionBridgePlugin};
use bevy::prelude::shape;
use bevy::{
    asset::AssetPlugin,
    core_pipeline::{clear_color::ClearColorConfig, core_3d::Camera3dBundle},
    math::primitives::Plane3d,
    prelude::*,
    render::camera::{PerspectiveProjection, Projection},
    window::{
        MonitorSelection, PresentMode, Window, WindowPlugin, WindowPosition, WindowResolution,
    },
    winit::WinitPlugin,
};
use clap::Parser;
use http_api::HttpApiPlugin;
use std::{
    net::SocketAddr,
    path::PathBuf,
    time::{Duration, Instant},
};
use tracing::Level;
use tracing_subscriber::fmt::time::UtcTime;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, value_name = "FILE", conflicts_with = "replay")]
    record: Option<PathBuf>,

    #[arg(long, value_name = "FILE", conflicts_with = "record")]
    replay: Option<PathBuf>,

    #[arg(long, value_name = "SECONDS")]
    duration: Option<f64>,

    #[arg(long, value_name = "ADDR:PORT", default_value = "127.0.0.1:8787")]
    bind: SocketAddr,
}

#[derive(Resource)]
struct TimedExit {
    start: Instant,
    duration: Duration,
}

fn main() {
    let cli = Cli::parse();

    let mode = match (cli.record, cli.replay) {
        (Some(path), None) => SimulationMode::Record { path },
        (None, Some(path)) => SimulationMode::Replay { path },
        (None, None) => SimulationMode::Normal,
        _ => unreachable!("clap conflicts ensure only one of record/replay is set"),
    };

    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(Level::INFO)
        .with_timer(UtcTime::rfc_3339())
        .init();

    let mut app = App::new();

    app.insert_resource(mode);

    if let Some(duration) = cli.duration {
        app.insert_resource(TimedExit {
            start: Instant::now(),
            duration: Duration::from_secs_f64(duration.max(0.0)),
        });
    }

    app.insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.07)));

    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        HierarchyPlugin,
        DiagnosticsPlugin,
        InputPlugin,
        AssetPlugin::default(),
        WindowPlugin {
            primary_window: Some(Window {
                title: "Nexus".into(),
                resolution: WindowResolution::new(1280.0, 720.0),
                present_mode: PresentMode::AutoVsync,
                position: WindowPosition::Centered(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        },
        WinitPlugin::default(),
        bevy::render::RenderPlugin::default(),
        bevy::core_pipeline::CorePipelinePlugin::default(),
        bevy::sprite::SpritePlugin::default(),
        bevy::pbr::PbrPlugin::default(),
        AnchorPlugin::default(),
        HttpApiPlugin::new(cli.bind),
        HudPlugin,
        PerceptionBridgePlugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, (exit_on_esc, exit_on_duration))
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 400.0,
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.05, 0.05, 0.08)),
            ..default()
        },
        projection: Projection::Perspective(PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            ..default()
        }),
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

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1.2,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(2.0, 3.0, 1.0),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Plane3d::default())),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.25, 0.28),
            perceptual_roughness: 0.9,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.7, 0.85, 1.0),
                perceptual_roughness: 0.5,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        })
        .insert(Name::new("cube_1"));
}

fn exit_on_esc(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

fn exit_on_duration(timer: Option<Res<TimedExit>>, mut exit: EventWriter<AppExit>) {
    if let Some(timer) = timer {
        if timer.start.elapsed() >= timer.duration {
            exit.send(AppExit::Success);
        }
    }
}
