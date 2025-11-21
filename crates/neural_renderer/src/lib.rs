//! Minimal neural renderer abstraction used by Nexus.
//!
//! This crate currently provides a mock backend that turns [`world_state::WorldSnapshot`]
//! data into a simple textual summary. It can also expose Bevy-friendly helper functions
//! for debug overlays when the optional `bevy` feature is enabled.

use std::fmt::Display;

use thiserror::Error;
use tracing::info;
use world_state::{Camera, Light, WorldEntity, WorldSnapshot};

#[cfg(feature = "bevy")]
use bevy::prelude::{
    AssetServer, Color, Commands, Component, Entity, PositionType, Style, Text, TextAlignment,
    TextBundle, TextStyle, Val,
};

pub type RenderResult<T> = Result<T, RenderError>;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("unsupported backend: {0}")]
    UnsupportedBackend(String),
    #[error("rendering failed: {0}")]
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderRequest {
    pub width: u32,
    pub height: u32,
    pub camera: Option<RenderCamera>,
    pub light: Option<RenderLight>,
    pub entities: Vec<RenderEntity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderCamera {
    pub translation: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderLight {
    pub color: [f32; 3],
    pub intensity: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderEntity {
    pub id: String,
    pub translation: [f32; 3],
    pub scale: [f32; 3],
    pub color: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderOutput {
    pub summary: String,
}

pub trait RendererBackend: Send + Sync + 'static {
    fn render(&mut self, request: RenderRequest) -> RenderResult<RenderOutput>;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RendererBackendKind {
    Mock,
}

impl Display for RendererBackendKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RendererBackendKind::Mock => write!(f, "mock"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeuralRendererConfig {
    pub backend: RendererBackendKind,
}

impl Default for NeuralRendererConfig {
    fn default() -> Self {
        Self {
            backend: RendererBackendKind::Mock,
        }
    }
}

pub fn build_renderer_from_config(
    config: &NeuralRendererConfig,
) -> RenderResult<Box<dyn RendererBackend>> {
    match config.backend {
        RendererBackendKind::Mock => Ok(Box::new(MockRenderer::default())),
    }
}

#[derive(Default)]
pub struct MockRenderer {
    rendered_frames: usize,
}

impl RendererBackend for MockRenderer {
    fn render(&mut self, request: RenderRequest) -> RenderResult<RenderOutput> {
        self.rendered_frames += 1;
        let entity_count = request.entities.len();
        let summary = format!(
            "[MockRenderer] frame {}: {} entities at {}x{}",
            self.rendered_frames, entity_count, request.width, request.height
        );
        info!(target: "neural_renderer", summary);
        Ok(RenderOutput { summary })
    }

    fn name(&self) -> &str {
        "mock"
    }
}

pub fn render_request_from_world(world: &WorldSnapshot, width: u32, height: u32) -> RenderRequest {
    RenderRequest {
        width,
        height,
        camera: world
            .camera
            .as_ref()
            .and_then(|camera| camera.translation)
            .map(|translation| RenderCamera { translation }),
        light: world.light.as_ref().map(|light| RenderLight {
            color: light.color.unwrap_or([1.0, 1.0, 1.0]),
            intensity: light.intensity.unwrap_or(1.0),
        }),
        entities: world
            .entities
            .iter()
            .map(|entity| RenderEntity {
                id: entity.id.clone(),
                translation: entity.transform.translation.unwrap_or([0.0, 0.0, 0.0]),
                scale: entity.transform.scale.unwrap_or([1.0, 1.0, 1.0]),
                color: entity.material.color.unwrap_or([1.0, 1.0, 1.0]),
            })
            .collect(),
    }
}

#[cfg(feature = "bevy")]
#[derive(Component)]
pub struct NeuralOverlayLabel;

/// Spawn a simple on-screen overlay that highlights when neural rendering is active.
///
/// Returns the spawned [`Entity`] so the caller can track or despawn it later.
#[cfg(feature = "bevy")]
pub fn spawn_debug_overlay(commands: &mut Commands, asset_server: &AssetServer) -> Entity {
    commands
        .spawn((
            NeuralOverlayLabel,
            TextBundle {
                text: Text::from_section(
                    "NEURAL MODE",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 28.0,
                        color: Color::srgb(1.0, 0.3, 0.6),
                    },
                )
                .with_alignment(TextAlignment::Left),
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(16.0),
                    top: Val::Px(16.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_world() -> WorldSnapshot {
        WorldSnapshot {
            entities: vec![WorldEntity {
                id: "a".to_string(),
                kind: Some("sprite".to_string()),
                transform: world_state::TransformData {
                    translation: Some([1.0, 2.0, 3.0]),
                    rotation: None,
                    scale: Some([2.0, 2.0, 1.0]),
                },
                material: world_state::MaterialData {
                    color: Some([0.2, 0.3, 0.4]),
                },
            }],
            camera: Some(Camera {
                translation: Some([0.0, 1.0, 5.0]),
            }),
            light: Some(Light {
                color: Some([0.1, 0.2, 0.3]),
                intensity: Some(0.7),
            }),
        }
    }

    #[test]
    fn build_request_from_world() {
        let world = build_test_world();
        let request = render_request_from_world(&world, 640, 480);

        assert_eq!(request.width, 640);
        assert_eq!(request.height, 480);
        assert_eq!(request.entities.len(), 1);
        assert_eq!(
            request.camera,
            Some(RenderCamera {
                translation: [0.0, 1.0, 5.0]
            })
        );
        assert_eq!(
            request.light,
            Some(RenderLight {
                color: [0.1, 0.2, 0.3],
                intensity: 0.7,
            })
        );
    }

    #[test]
    fn mock_renderer_counts_frames() -> anyhow::Result<()> {
        let mut renderer = MockRenderer::default();
        let world = build_test_world();
        let request = render_request_from_world(&world, 800, 600);

        let output = renderer.render(request.clone())?;
        assert!(output.summary.contains("frame 1"));
        let output2 = renderer.render(request)?;
        assert!(output2.summary.contains("frame 2"));
        Ok(())
    }
}
