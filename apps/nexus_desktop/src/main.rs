use anchor::{AnchorPlugin, Velocity};
use anyhow::Result;
use bevy::app::App;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowPlugin};
use neural_renderer::{
    build_renderer_from_config, render_request_from_world, NeuralRendererConfig, RendererBackend,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tracing::info;
use world_state::{Collider, WorldSnapshot};

const WORLD_PATH: &str = "apps/nexus_desktop/assets/world.json";

#[derive(Resource)]
struct WorldSyncState {
    path: PathBuf,
    last_modified: Option<SystemTime>,
    timer: Timer,
    latest_snapshot: Option<WorldSnapshot>,
}

#[derive(Resource)]
struct NeuralRendererState {
    renderer: Box<dyn RendererBackend>,
    timer: Timer,
}

#[derive(Component)]
struct WorldEntityId(String);

impl WorldSyncState {
    fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            last_modified: None,
            timer: Timer::new(Duration::from_millis(100), TimerMode::Repeating),
            latest_snapshot: None,
        }
    }

    fn read_snapshot(&mut self) -> Option<WorldSnapshot> {
        let metadata = match fs::metadata(&self.path) {
            Ok(m) => m,
            Err(e) => {
                info!("Failed to read metadata for world.json: {}", e);
                return None;
            }
        };
        let modified = metadata.modified().ok();

        if let (Some(last), Some(current)) = (self.last_modified, modified) {
            if current <= last {
                return None;
            }
        }

        let raw = match fs::read_to_string(&self.path) {
            Ok(s) => s,
            Err(e) => {
                info!("Failed to read world.json: {}", e);
                return None;
            }
        };
        
        match serde_json::from_str::<WorldSnapshot>(&raw) {
            Ok(mut parsed) => {
                // Debug Default: If world is empty, spawn a visible debug cube
                if parsed.entities.is_empty() {
                    info!("World is empty. Injecting default debug cube.");
                    parsed.entities.push(world_state::WorldEntity {
                        id: "debug_cube".to_string(),
                        kind: Some("cube".to_string()),
                        transform: world_state::TransformData {
                            translation: Some([0.0, 2.0, 0.0]),
                            rotation: Some([0.0, 0.0, 0.0]),
                            scale: Some([1.0, 1.0, 1.0]),
                        },
                        material: world_state::MaterialData {
                            color: Some([1.0, 0.0, 1.0]), // Magenta
                        },
                    });
                }

                info!("Successfully loaded world.json with {} entities", parsed.entities.len());
                self.last_modified = modified;
                self.latest_snapshot = Some(parsed.clone());
                Some(parsed)
            },
            Err(e) => {
                info!("Failed to parse world.json: {}", e);
                info!("Raw content snippet: {:.100}", raw);
                None
            }
        }
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!(target: "nexus_desktop", "Launching Nexus desktop app");

    let base_clear = Color::srgb(0.02, 0.02, 0.08);

    App::new()
        .insert_resource(ClearColor(base_clear))
        .insert_resource(WorldSyncState::new(WORLD_PATH))
        .insert_resource(build_renderer_resource())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Nexus Engine".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }).disable::<bevy::log::LogPlugin>())
        .add_plugins(AnchorPlugin)
        .add_systems(Startup, (setup_scene, log_startup))
        .add_systems(Update, (sync_world_file, run_neural_renderer))
        .run();

    Ok(())
}

fn log_startup() {
    info!("Nexus desktop app startup: Bevy app running, waiting for window events");
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn sync_world_file(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<WorldSyncState>,
    existing_entities: Query<(Entity, &WorldEntityId)>,
    mut sprite_query: Query<(&mut Transform, &mut Sprite), (With<WorldEntityId>, Without<Camera>)>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut clear_color: ResMut<ClearColor>,
) {
    state.timer.tick(time.delta());
    if !state.timer.finished() {
        return;
    }

    let Some(world) = state.read_snapshot() else {
        return;
    };

    let mut entity_map: HashMap<String, Entity> = existing_entities
        .iter()
        .map(|(entity, id)| (id.0.clone(), entity))
        .collect();

    for entity_data in world.entities.clone() {
        if entity_data.id.is_empty() {
            continue;
        }

        let translation = entity_data.transform.translation.unwrap_or([0.0, 0.0, 0.0]);
        let scale = entity_data.transform.scale.unwrap_or([1.0, 1.0, 1.0]);
        let color_arr = entity_data.material.color.unwrap_or([1.0, 1.0, 1.0]);
        let color = Color::srgb(color_arr[0], color_arr[1], color_arr[2]);

        // Force Z=0.0 for all sprites so they are always in front of the background.
        let z_pos = 0.0;

        if let Some(existing_entity) = entity_map.remove(&entity_data.id) {
            if let Ok((mut transform, mut sprite)) = sprite_query.get_mut(existing_entity) {
                // SCALE FIX: Multiply position by 50.0 to visualize small units on screen
                transform.translation = Vec3::new(translation[0] * 50.0, translation[1] * 50.0, z_pos);
                transform.scale = Vec3::new(scale[0], scale[1], scale[2]);
                sprite.color = color;
            }
        } else {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::splat(60.0)),
                        ..Default::default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        translation[0] * 50.0,
                        translation[1] * 50.0,
                        z_pos,
                    ))
                    .with_scale(Vec3::new(scale[0], scale[1], scale[2])),
                    ..Default::default()
                },
                WorldEntityId(entity_data.id.clone()),
                Velocity(Vec3::ZERO),
                Collider {
                    half_extents: Vec3::new(30.0, 30.0, 0.0),
                },
            ));
        }
    }

    for entity in entity_map.values() {
        commands.entity(*entity).despawn_recursive();
    }

    if let Some(camera_data) = world.camera.clone() {
        if let Some(translation) = camera_data.translation {
            if let Some(mut camera_transform) = camera_query.iter_mut().next() {
                // Ignore world.json camera Z for 2D orthographic view, keep it at default
                camera_transform.translation =
                    Vec3::new(translation[0] * 50.0, translation[1] * 50.0, 999.9);
            }
        }
    }

    // For now, ignore light color for ClearColor to keep a stable dark background.
}

fn build_renderer_resource() -> NeuralRendererState {
    let config = NeuralRendererConfig::default();
    let renderer =
        build_renderer_from_config(&config).expect("failed to construct neural renderer backend");
    NeuralRendererState {
        renderer,
        timer: Timer::from_seconds(0.5, TimerMode::Repeating),
    }
}

fn run_neural_renderer(
    time: Res<Time>,
    mut state: ResMut<NeuralRendererState>,
    world_state: Res<WorldSyncState>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    state.timer.tick(time.delta());
    if !state.timer.finished() {
        return;
    }

    let Some(world) = world_state.latest_snapshot.clone() else {
        return;
    };

    let (width, height) = windows
        .get_single()
        .map(|window| (window.width() as u32, window.height() as u32))
        .unwrap_or((800, 600));

    let request = render_request_from_world(&world, width, height);
    match state.renderer.render(request) {
        Ok(output) => {
            info!(target: "nexus_desktop", "Neural renderer output: {}", output.summary);
        }
        Err(err) => {
            info!(target: "nexus_desktop", "Neural renderer failed: {err}");
        }
    }
}


