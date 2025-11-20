use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use bevy::app::AppExit;
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

#[derive(Event, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Intent {
    pub verb: String,
    pub args: Value,
}

#[derive(Event, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InputEvent {
    pub tick: u64,
    pub action: String,
    pub data: Value,
}

#[derive(Resource, Debug, Clone)]
pub enum SimulationMode {
    Normal,
    Record { path: PathBuf },
    Replay { path: PathBuf },
}

impl Default for SimulationMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct SimulationTick(u64);

#[derive(Resource)]
struct RecordWriter {
    writer: BufWriter<File>,
}

#[derive(Resource)]
struct ReplayState {
    frames: VecDeque<RecordedFrame>,
    final_tick: u64,
    exit_requested: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordedFrame {
    tick: u64,
    intents: Vec<Intent>,
    input_events: Vec<InputEvent>,
}

impl RecordWriter {
    fn new(path: &Path) -> Result<Self> {
        let file = File::create(path)
            .with_context(|| format!("failed to create record file at {}", path.display()))?;
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    fn write_frame(&mut self, frame: &RecordedFrame) -> Result<()> {
        serde_json::to_writer(&mut self.writer, frame)
            .context("failed to serialize recorded frame")?;
        self.writer
            .write_all(b"\n")
            .context("failed to write newline")?;
        self.writer.flush().context("failed to flush record file")?;
        Ok(())
    }
}

impl ReplayState {
    fn from_path(path: &Path) -> Result<Self> {
        let file = File::open(path)
            .with_context(|| format!("failed to open replay file at {}", path.display()))?;
        let reader = BufReader::new(file);

        let mut frames: VecDeque<RecordedFrame> = VecDeque::new();
        for (index, line) in reader.lines().enumerate() {
            let line = line.context("failed to read line from replay file")?;
            if line.trim().is_empty() {
                continue;
            }

            let frame: RecordedFrame = serde_json::from_str(&line)
                .with_context(|| format!("failed to parse replay frame at line {}", index + 1))?;

            if let Some(previous) = frames.back() {
                if frame.tick < previous.tick {
                    warn!(
                        current_tick = frame.tick,
                        previous_tick = previous.tick,
                        "replay frame tick decreased; playback order may be incorrect"
                    );
                }
            }

            frames.push_back(frame);
        }

        let final_tick = frames.back().map(|frame| frame.tick).unwrap_or(0);

        Ok(Self {
            frames,
            final_tick,
            exit_requested: false,
        })
    }
}

fn initialize_mode(app: &mut App) -> SimulationMode {
    let world = app.world_mut();
    if !world.contains_resource::<SimulationMode>() {
        world.insert_resource(SimulationMode::default());
    }
    world.resource::<SimulationMode>().clone()
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
        let mode = initialize_mode(app);

        if let SimulationMode::Record { path } = &mode {
            let writer = RecordWriter::new(path)
                .unwrap_or_else(|err| panic!("failed to initialize record writer: {err:?}"));
            app.insert_resource(writer);
        }

        if let SimulationMode::Replay { path } = &mode {
            let state = ReplayState::from_path(path)
                .unwrap_or_else(|err| panic!("failed to load replay data: {err:?}"));
            app.insert_resource(state);
        }

        metrics::init_metrics(app);

        app.insert_resource(SimulationTick::default())
            .register_type::<Velocity>()
            .register_type::<BodySize>()
            .add_event::<Intent>()
            .add_event::<InputEvent>()
            .configure_sets(
                FixedUpdate,
                AnchorSystemSet::ApplyIntents.before(AnchorSystemSet::Integrate),
            )
            .add_systems(FixedFirst, advance_tick)
            .add_systems(FixedFirst, inject_replay_events.after(advance_tick))
            .add_systems(
                FixedUpdate,
                convert_input_events_to_intents.in_set(AnchorSystemSet::ApplyIntents),
            )
            .add_systems(
                FixedUpdate,
                apply_intents
                    .in_set(AnchorSystemSet::ApplyIntents)
                    .after(convert_input_events_to_intents),
            )
            .add_systems(
                FixedUpdate,
                integrate_velocity.in_set(AnchorSystemSet::Integrate),
            )
            .add_systems(FixedLast, record_events)
            .add_systems(FixedLast, monitor_replay_completion.after(record_events))
            .add_systems(PostUpdate, report_checksum_on_exit);
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

    let mesh = meshes.add(Mesh::from(Cuboid::new(size.x, size.y, size.z)));
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

fn advance_tick(mut tick: ResMut<SimulationTick>) {
    **tick += 1;
}

fn inject_replay_events(
    tick: Res<SimulationTick>,
    mut state: Option<ResMut<ReplayState>>,
    mut intents: EventWriter<Intent>,
    mut inputs: EventWriter<InputEvent>,
) {
    let Some(mut state) = state else {
        return;
    };

    let current_tick = **tick;

    while let Some(frame) = state.frames.front() {
        if frame.tick > current_tick {
            break;
        }

        let mut frame = state.frames.pop_front().expect("frame exists");
        if frame.tick < current_tick {
            warn!(
                frame_tick = frame.tick,
                current_tick, "replay frame tick is earlier than current tick"
            );
        }

        for intent in frame.intents.drain(..) {
            intents.send(intent);
        }

        for mut input in frame.input_events.drain(..) {
            if input.tick != frame.tick {
                warn!(
                    expected_tick = frame.tick,
                    actual_tick = input.tick,
                    action = %input.action,
                    "input event tick mismatch in replay; overriding"
                );
                input.tick = frame.tick;
            }
            inputs.send(input);
        }
    }
}

fn convert_input_events_to_intents(
    tick: Res<SimulationTick>,
    mut reader: EventReader<InputEvent>,
    mut writer: EventWriter<Intent>,
) {
    let current_tick = **tick;
    for event in reader.read() {
        if event.tick != current_tick {
            warn!(
                expected_tick = current_tick,
                actual_tick = event.tick,
                action = %event.action,
                "received input event with mismatched tick"
            );
        }

        writer.send(Intent {
            verb: event.action.clone(),
            args: event.data.clone(),
        });
    }
}

fn record_events(
    tick: Res<SimulationTick>,
    mut recorder: Option<ResMut<RecordWriter>>,
    mut intent_reader: EventReader<Intent>,
    mut input_reader: EventReader<InputEvent>,
) {
    let Some(mut recorder) = recorder else {
        return;
    };

    let current_tick = **tick;

    let mut input_events: Vec<InputEvent> = input_reader
        .read()
        .cloned()
        .map(|mut input| {
            if input.tick != current_tick {
                warn!(
                    expected_tick = current_tick,
                    actual_tick = input.tick,
                    action = %input.action,
                    "recording input event with mismatched tick; overriding"
                );
                input.tick = current_tick;
            }
            input
        })
        .collect();

    let intents: Vec<Intent> = intent_reader
        .read()
        .cloned()
        .filter(|intent| {
            !input_events.iter().any(|input| {
                input.tick == current_tick
                    && input.action == intent.verb
                    && input.data == intent.args
            })
        })
        .collect();

    let frame = RecordedFrame {
        tick: current_tick,
        intents,
        input_events,
    };

    if let Err(err) = recorder.write_frame(&frame) {
        warn!(?err, "failed to write record frame");
    }
}

fn monitor_replay_completion(
    tick: Res<SimulationTick>,
    mut state: Option<ResMut<ReplayState>>,
    mut exit_writer: EventWriter<AppExit>,
) {
    let Some(mut state) = state else {
        return;
    };

    if state.exit_requested {
        return;
    }

    if !state.frames.is_empty() {
        return;
    }

    if **tick < state.final_tick {
        return;
    }

    state.exit_requested = true;
    exit_writer.send(AppExit::default());
}

fn report_checksum_on_exit(
    mode: Option<Res<SimulationMode>>,
    mut exit_events: EventReader<AppExit>,
    query: Query<&Transform>,
) {
    let Some(mode) = mode else {
        return;
    };

    if !matches!(&*mode, SimulationMode::Replay { .. }) {
        return;
    }

    if exit_events.read().next().is_none() {
        return;
    }

    let mut sum = Vec3::ZERO;
    for transform in &query {
        sum += transform.translation;
    }

    println!("Replay checksum: {:.6} {:.6} {:.6}", sum.x, sum.y, sum.z);
}
