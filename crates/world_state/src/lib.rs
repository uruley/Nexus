use bevy::prelude::*;

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
