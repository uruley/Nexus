use bevy::prelude::*;
use bevy::time::Fixed;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn};

mod metrics;

pub use metrics::FrameTimings;

pub const INTENT_SPAWN: &str = "Spawn";
pub const INTENT_MOVE: &str = "Move";
pub const INTENT_APPLY_FORCE: &str = "ApplyForce";
pub const INTENT_DESPAWN: &str = "Despawn";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnArgs {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub size: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoveArgs {
    pub entity: u64,
    pub vel: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyForceArgs {
    pub entity: u64,
    pub impulse: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DespawnArgs {
    pub entity: u64,
}

#[derive(Event, Debug, Clone)]
pub struct Intent {
    pub verb: String,
    pub args: Value,
}

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct Velocity(pub Vec3);

impl Default for Velocity {
    fn default() -> Self {
        Self(Vec3::ZERO)
    }
}

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct BodySize(pub Vec3);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnchorSystemSet {
    ApplyIntents,
    Integrate,
}

#[derive(Default)]
pub struct AnchorPlugin;

impl Plugin for AnchorPlugin {
    fn build(&self, app: &mut App) {
        metrics::init_metrics(app);

        app.register_type::<Velocity>()
            .register_type::<BodySize>()
            .add_event::<Intent>()
            .configure_sets(
                FixedUpdate,
                AnchorSystemSet::ApplyIntents.before(AnchorSystemSet::Integrate),
            )
            .add_systems(
                FixedUpdate,
                apply_intents.in_set(AnchorSystemSet::ApplyIntents),
            )
            .add_systems(
                FixedUpdate,
                integrate_velocity.in_set(AnchorSystemSet::Integrate),
            );
    }
}

fn apply_intents(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut reader: EventReader<Intent>,
    mut velocities: Query<&mut Velocity>,
) {
    for intent in reader.read() {
        match intent.verb.as_str() {
            INTENT_SPAWN => match serde_json::from_value::<SpawnArgs>(intent.args.clone()) {
                Ok(args) => spawn_entity(&mut commands, &mut meshes, &mut materials, &args),
                Err(err) => warn!("invalid spawn args: {err}"),
            },
            INTENT_MOVE => match serde_json::from_value::<MoveArgs>(intent.args.clone()) {
                Ok(args) => set_velocity(&mut velocities, args.entity, Vec3::from_array(args.vel)),
                Err(err) => warn!("invalid move args: {err}"),
            },
            INTENT_APPLY_FORCE => {
                match serde_json::from_value::<ApplyForceArgs>(intent.args.clone()) {
                    Ok(args) => {
                        add_velocity(&mut velocities, args.entity, Vec3::from_array(args.impulse))
                    }
                    Err(err) => warn!("invalid apply force args: {err}"),
                }
            }
            INTENT_DESPAWN => match serde_json::from_value::<DespawnArgs>(intent.args.clone()) {
                Ok(args) => despawn_entity(&mut commands, args.entity),
                Err(err) => warn!("invalid despawn args: {err}"),
            },
            other => warn!("unknown intent verb received: {other}"),
        }
    }
}

fn spawn_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    args: &SpawnArgs,
) {
    let position = Vec3::from_array(args.pos);
    let velocity = Vec3::from_array(args.vel);
    let size = Vec3::from_array(args.size);

    let mesh = meshes.add(Mesh::from(bevy::render::mesh::shape::Box::new(
        size.x, size.y, size.z,
    )));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.7, 1.0),
        ..default()
    });

    commands
        .spawn(PbrBundle {
            mesh,
            material,
            transform: Transform::from_translation(position),
            ..default()
        })
        .insert(Velocity(velocity))
        .insert(BodySize(size));

    info!(?position, ?velocity, ?size, "spawned cube entity");
}

fn set_velocity(velocities: &mut Query<&mut Velocity>, entity_bits: u64, velocity: Vec3) {
    let entity = Entity::from_bits(entity_bits);
    match velocities.get_mut(entity) {
        Ok(mut vel) => {
            vel.0 = velocity;
            info!(entity = %entity.index(), ?velocity, "set velocity");
        }
        Err(_) => warn!(entity = entity_bits, "move intent for unknown entity"),
    }
}

fn add_velocity(velocities: &mut Query<&mut Velocity>, entity_bits: u64, impulse: Vec3) {
    let entity = Entity::from_bits(entity_bits);
    match velocities.get_mut(entity) {
        Ok(mut vel) => {
            vel.0 += impulse;
            info!(entity = %entity.index(), ?impulse, "applied impulse");
        }
        Err(_) => warn!(
            entity = entity_bits,
            "apply_force intent for unknown entity"
        ),
    }
}

fn despawn_entity(commands: &mut Commands, entity_bits: u64) {
    let entity = Entity::from_bits(entity_bits);
    if commands.get_entity(entity).is_some() {
        commands.entity(entity).despawn_recursive();
        info!(entity = %entity.index(), "despawned entity");
    } else {
        warn!(entity = entity_bits, "despawn intent for unknown entity");
    }
}

fn integrate_velocity(time: Res<Time<Fixed>>, mut query: Query<(&mut Transform, &Velocity)>) {
    let delta = time.delta_seconds();
    if delta == 0.0 {
        return;
    }

    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0 * delta;
    }
}
