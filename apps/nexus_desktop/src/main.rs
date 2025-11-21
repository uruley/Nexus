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
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tracing::info;
use world_state::Collider;

const WORLD_PATH: &str = "apps/nexus_desktop/assets/world.json";

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct TransformData {
    translation: Option<[f32; 3]>,
    rotation: Option<[f32; 3]>,
    scale: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct MaterialData {
    color: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct EntityData {
    id: String,
    kind: Option<String>,
    transform: Option<TransformData>,
    material: Option<MaterialData>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct CameraData {
    translation: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct LightData {
    color: Option<[f32; 3]>,
    intensity: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct WorldFile {
    entities: Vec<EntityData>,
    camera: Option<CameraData>,
    light: Option<LightData>,
}

#[derive(Resource)]
struct WorldSyncState {
    path: PathBuf,
    last_modified: Option<SystemTime>,
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
        }
    }

    fn read_snapshot(&mut self) -> Option<WorldFile> {
        let metadata = fs::metadata(&self.path).ok()?;
        let modified = metadata.modified().ok();

        if let (Some(last), Some(current)) = (self.last_modified, modified) {
            if current <= last {
                return None;
            }
        }

        let raw = fs::read_to_string(&self.path).ok()?;
        let parsed: WorldFile = serde_json::from_str(&raw).ok()?;
        self.last_modified = modified;
        Some(parsed)
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!(target: "nexus_desktop", "Launching Nexus desktop app");

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.08)))
        .insert_resource(WorldSyncState::new(WORLD_PATH))
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
        .add_systems(Update, sync_world_file)
        .run();

    Ok(())
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn sync_world_file(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<WorldSyncState>,
    existing_entities: Query<(Entity, &WorldEntityId)>,
    mut sprite_query: Query<(&mut Transform, &mut Sprite), With<WorldEntityId>>,
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

    for entity_data in world.entities {
        if entity_data.id.is_empty() {
            continue;
        }

        let translation = entity_data
            .transform
            .as_ref()
            .and_then(|t| t.translation)
            .unwrap_or([0.0, 0.0, 0.0]);
        let scale = entity_data
            .transform
            .as_ref()
            .and_then(|t| t.scale)
            .unwrap_or([1.0, 1.0, 1.0]);
        let color_arr = entity_data
            .material
            .as_ref()
            .and_then(|m| m.color)
            .unwrap_or([1.0, 1.0, 1.0]);
        let color = Color::srgb(color_arr[0], color_arr[1], color_arr[2]);

        if let Some(existing_entity) = entity_map.remove(&entity_data.id) {
            if let Ok((mut transform, mut sprite)) = sprite_query.get_mut(existing_entity) {
                transform.translation = Vec3::new(translation[0], translation[1], translation[2]);
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
                        translation[0],
                        translation[1],
                        translation[2],
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

    if let Some(camera_data) = world.camera {
        if let Some(translation) = camera_data.translation {
            if let Some(mut camera_transform) = camera_query.iter_mut().next() {
                camera_transform.translation = Vec3::new(
                    translation[0],
                    translation[1],
                    translation[2],
                );
            }
        }
    }

    if let Some(light) = world.light {
        if let Some(color) = light.color {
            clear_color.0 = Color::srgb(color[0], color[1], color[2]);
        }

        if let Some(intensity) = light.intensity {
            let clamped = intensity.clamp(0.0, 5.0);
            clear_color.0.set_a((clamped / 5.0).clamp(0.1, 1.0));
        }
    }
}
