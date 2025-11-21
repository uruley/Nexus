use bevy::prelude::*;
use serde::Deserialize;

/// Full dimensions for rendering meshes or sprites.
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct BodySize(pub Vec3);

/// Axis-aligned bounding box used for simple physics interactions.
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct Collider {
    pub half_extents: Vec3,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct TransformData {
    pub translation: Option<[f32; 3]>,
    pub rotation: Option<[f32; 3]>,
    pub scale: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct MaterialData {
    pub color: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct WorldEntity {
    pub id: String,
    pub kind: Option<String>,
    pub transform: TransformData,
    pub material: MaterialData,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct Camera {
    pub translation: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct Light {
    pub color: Option<[f32; 3]>,
    pub intensity: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct WorldSnapshot {
    pub entities: Vec<WorldEntity>,
    pub camera: Option<Camera>,
    pub light: Option<Light>,
}
