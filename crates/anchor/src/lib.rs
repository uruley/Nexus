use bevy::prelude::*;
use world_state::Collider;

const GRAVITY: f32 = 9.81;

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct Velocity(pub Vec3);

pub struct AnchorPlugin;

impl Plugin for AnchorPlugin {
    fn build(&self, app: &mut App) {
        // Register the physics types and systems for the anchor world.
        app.register_type::<Velocity>()
            .register_type::<Collider>()
            .add_systems(
                Update,
                (apply_gravity, integrate_velocity, clamp_to_floor).chain(),
            );
    }
}

fn apply_gravity(time: Res<Time>, mut query: Query<&mut Velocity, With<Collider>>) {
    let dt = time.delta_seconds();

    for mut velocity in &mut query {
        velocity.0.y -= GRAVITY * dt;
    }
}

fn integrate_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    let dt = time.delta_seconds();

    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0 * dt;
    }
}

fn clamp_to_floor(mut query: Query<(&mut Transform, &mut Velocity, &Collider)>) {
    for (mut transform, mut velocity, collider) in &mut query {
        let half_height = collider.half_extents.y;
        let bottom = transform.translation.y - half_height;

        if bottom < 0.0 {
            transform.translation.y = half_height;
            velocity.0.y = 0.0;
        }
    }
}
