use anchor::{AnchorPlugin, Velocity};
use anyhow::Result;
use bevy::app::App;
use bevy::asset::AssetPlugin;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::core_pipeline::CorePipelinePlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::sprite::SpritePlugin;
use bevy::text::TextPlugin;
use bevy::transform::TransformPlugin;
use bevy::ui::UiPlugin;
use bevy::window::{PrimaryWindow, WindowPlugin};
use bevy::winit::WinitPlugin;
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

#[derive(Resource, Default)]
struct DebugHudState {
    entity_count: usize,
    last_command: Option<String>,
}

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

#[derive(Component)]
struct HudText;

#[derive(Event)]
struct RouterCommandEvent {
    description: String,
}

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
        let metadata = fs::metadata(&self.path).ok()?;
        let modified = metadata.modified().ok();

        if let (Some(last), Some(current)) = (self.last_modified, modified) {
            if current <= last {
                return None;
            }
        }

        let raw = fs::read_to_string(&self.path).ok()?;
        let parsed: WorldSnapshot = serde_json::from_str(&raw).ok()?;
        self.last_modified = modified;
        self.latest_snapshot = Some(parsed.clone());
        Some(parsed)
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!(target: "nexus_desktop", "Launching Nexus desktop app");

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.08)))
        .insert_resource(WorldSyncState::new(WORLD_PATH))
        .insert_resource(build_renderer_resource())
        .init_resource::<DebugHudState>()
        .add_event::<RouterCommandEvent>()
        .add_plugins((
            MinimalPlugins,
            WindowPlugin::default(),
            WinitPlugin::default(),
            AssetPlugin::default(),
            TransformPlugin,
            RenderPlugin::default(),
            CorePipelinePlugin::default(),
            SpritePlugin::default(),
            TextPlugin::default(),
            UiPlugin::default(),
        ))
        .add_plugins(AnchorPlugin)
        .add_systems(Startup, (setup_scene, setup_hud))
        .add_systems(
            Update,
            (sync_world_file, run_neural_renderer, update_hud_text),
        )
        .add_systems(Update, capture_router_commands)
        .run();

    Ok(())
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.default_font();

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            z_index: ZIndex::Global(10),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new(
                        "Entities: 0\n",
                        TextStyle {
                            font: font.clone(),
                            font_size: 16.0,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    ),
                    TextSection::new(
                        "Last command: (none)",
                        TextStyle {
                            font,
                            font_size: 16.0,
                            color: Color::GRAY,
                            ..Default::default()
                        },
                    ),
                ]),
                HudText,
            ));
        });
}

fn sync_world_file(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<WorldSyncState>,
    existing_entities: Query<(Entity, &WorldEntityId)>,
    mut sprite_query: Query<(&mut Transform, &mut Sprite), With<WorldEntityId>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut clear_color: ResMut<ClearColor>,
    mut hud_state: ResMut<DebugHudState>,
) {
    state.timer.tick(time.delta());
    if !state.timer.finished() {
        return;
    }

    let Some(world) = state.read_snapshot() else {
        return;
    };

    hud_state.entity_count = world.entities.len();

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

    if let Some(camera_data) = world.camera.clone() {
        if let Some(translation) = camera_data.translation {
            if let Some(mut camera_transform) = camera_query.iter_mut().next() {
                camera_transform.translation =
                    Vec3::new(translation[0], translation[1], translation[2]);
            }
        }
    }

    if let Some(light) = world.light.clone() {
        if let Some(color) = light.color {
            clear_color.0 = Color::srgb(color[0], color[1], color[2]);
        }

        if let Some(intensity) = light.intensity {
            let clamped = intensity.clamp(0.0, 5.0);
            clear_color.0.set_a((clamped / 5.0).clamp(0.1, 1.0));
        }
    }
}

fn capture_router_commands(
    mut events: EventReader<RouterCommandEvent>,
    mut hud_state: ResMut<DebugHudState>,
) {
    for event in events.read() {
        hud_state.last_command = Some(event.description.clone());
    }
}

fn update_hud_text(state: Res<DebugHudState>, mut query: Query<&mut Text, With<HudText>>) {
    if !state.is_changed() {
        return;
    }

    let last_command = state
        .last_command
        .as_ref()
        .map(|cmd| cmd.as_str())
        .unwrap_or("(none)");

    for mut text in &mut query {
        if let Some(first) = text.sections.get_mut(0) {
            first.value = format!("Entities: {}\n", state.entity_count);
        }

        if let Some(second) = text.sections.get_mut(1) {
            second.value = format!("Last command: {last_command}");
            second.style.color = if state.last_command.is_some() {
                Color::WHITE
            } else {
                Color::GRAY
            };
        }
    }
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
