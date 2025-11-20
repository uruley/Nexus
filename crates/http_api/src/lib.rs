use std::{
    collections::{HashMap, HashSet, VecDeque},
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use anchor::{
    ApplyForceArgs, DespawnArgs, Intent, MoveArgs, SpawnArgs, INTENT_APPLY_FORCE, INTENT_DESPAWN,
    INTENT_MOVE, INTENT_SPAWN,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use bevy::prelude::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use world_state::{self, Checksum, Diff as WorldDiff, EntitySnapshot, Snapshot as WorldSnapshot};

pub struct HttpApiPlugin {
    bind_addr: SocketAddr,
}

impl HttpApiPlugin {
    pub fn new(bind_addr: SocketAddr) -> Self {
        Self { bind_addr }
    }
}

impl Default for HttpApiPlugin {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:8787".parse().expect("default bind addr"),
        }
    }
}

impl Plugin for HttpApiPlugin {
    fn build(&self, app: &mut App) {
        let shared_state = SharedWorldState::default();
        let (sender, receiver) = unbounded();

        start_server(shared_state.clone(), sender.clone(), self.bind_addr);

        app.insert_resource(IntentReceiver { receiver })
            .insert_resource(shared_state)
            .insert_resource(IntentSender { sender })
            .add_systems(PreUpdate, pump_intents)
            .add_systems(PostUpdate, sync_world_state);
    }
}

#[derive(Resource, Clone)]
struct SharedWorldState {
    inner: Arc<RwLock<WorldStateStore>>,
}

impl Default for SharedWorldState {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(WorldStateStore::default())),
        }
    }
}

#[derive(Resource)]
struct IntentReceiver {
    receiver: Receiver<Intent>,
}

#[derive(Resource, Clone)]
struct IntentSender {
    sender: Sender<Intent>,
}

#[derive(Clone)]
struct ServerState {
    intents: Sender<Intent>,
    world: Arc<RwLock<WorldStateStore>>,
}

const HISTORY_LIMIT: usize = 1024;

struct WorldStateStore {
    tick: u64,
    checksum: Checksum,
    entities: HashMap<u64, EntitySnapshot>,
    history: VecDeque<DiffEntry>,
}

impl Default for WorldStateStore {
    fn default() -> Self {
        Self {
            tick: 0,
            checksum: world_state::checksum_for_state(0, &[]),
            entities: HashMap::new(),
            history: VecDeque::new(),
        }
    }
}

#[derive(Clone, Serialize, PartialEq)]
struct DiffEntry {
    tick: u64,
    base: Checksum,
    checksum: Checksum,
    added: Vec<EntitySnapshot>,
    removed: Vec<u64>,
    changed: Vec<EntitySnapshot>,
}

#[derive(Deserialize)]
struct IntentPayload {
    verb: String,
    args: Value,
}

#[derive(Deserialize)]
struct WorldDiffQuery {
    since: Option<String>,
}

enum DiffError {
    MissingSince,
    ParseError,
    UnknownChecksum,
    ChecksumTooOld,
}

impl DiffError {
    fn message(&self) -> &'static str {
        match self {
            DiffError::MissingSince => "missing since query parameter",
            DiffError::ParseError => "invalid checksum format",
            DiffError::UnknownChecksum => "unknown checksum",
            DiffError::ChecksumTooOld => "requested checksum is too old",
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct AcceptedResponse {
    status: &'static str,
}

fn start_server(world: SharedWorldState, sender: Sender<Intent>, bind_addr: SocketAddr) {
    let server_state = ServerState {
        intents: sender,
        world: world.inner.clone(),
    };

    let runtime = tokio::runtime::Runtime::new().expect("create tokio runtime");
    runtime.spawn(async move {
        if let Err(err) = run_server(server_state, bind_addr).await {
            error!("http server error: {err}");
        }
    });

    std::mem::forget(runtime);
}

async fn run_server(state: ServerState, bind_addr: SocketAddr) -> Result<(), anyhow::Error> {
    let router = Router::new()
        .route("/world/snapshot", get(get_world_snapshot))
        .route("/world/diff", get(get_world_diff))
        .route("/intent", post(post_intent))
        .with_state(state);

    let listener = TcpListener::bind(bind_addr).await?;
    info!("HTTP API listening on {bind_addr}");
    axum::serve(listener, router).await?;
    Ok(())
}

fn pump_intents(mut receiver: ResMut<IntentReceiver>, mut writer: EventWriter<Intent>) {
    loop {
        match receiver.receiver.try_recv() {
            Ok(intent) => {
                let _ = writer.send(intent);
            }
            Err(crossbeam_channel::TryRecvError::Empty) => break,
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                warn!("intent channel disconnected");
                break;
            }
        }
    }
}

fn sync_world_state(
    shared: Res<SharedWorldState>,
    query: Query<(Entity, &Transform, &anchor::Velocity, &anchor::BodySize)>,
) {
    let mut entities = HashMap::new();
    for (entity, transform, velocity, size) in &query {
        entities.insert(
            entity.to_bits(),
            EntitySnapshot {
                id: entity.to_bits(),
                pos: transform.translation.to_array(),
                vel: velocity.0.to_array(),
                size: size.0.to_array(),
            },
        );
    }

    let mut store = shared.inner.write().expect("world state lock");
    store.update(entities);
}

impl WorldStateStore {
    fn update(&mut self, new_entities: HashMap<u64, EntitySnapshot>) {
        let next_tick = self.tick + 1;
        let base_checksum = self.checksum;

        let mut added = Vec::new();
        let mut changed = Vec::new();
        let mut removed = Vec::new();

        for (&id, snapshot) in &new_entities {
            match self.entities.get(&id) {
                None => added.push(snapshot.clone()),
                Some(existing) if existing != snapshot => changed.push(snapshot.clone()),
                _ => {}
            }
        }

        for id in self.entities.keys() {
            if !new_entities.contains_key(id) {
                removed.push(*id);
            }
        }

        added.sort_by_key(|e| e.id);
        changed.sort_by_key(|e| e.id);
        removed.sort_unstable();

        let mut sorted_entities: Vec<_> = new_entities.values().cloned().collect();
        sorted_entities.sort_by_key(|e| e.id);
        let checksum = world_state::checksum_for_state(next_tick, &sorted_entities);

        self.tick = next_tick;
        self.checksum = checksum;
        self.entities = new_entities;
        self.history.push_back(DiffEntry {
            tick: next_tick,
            base: base_checksum,
            checksum,
            added,
            removed,
            changed,
        });

        while self.history.len() > HISTORY_LIMIT {
            self.history.pop_front();
        }
    }

    fn snapshot(&self) -> WorldSnapshot {
        let mut entities: Vec<_> = self.entities.values().cloned().collect();
        entities.sort_by_key(|e| e.id);
        WorldSnapshot {
            tick: self.tick,
            checksum: self.checksum,
            entities,
        }
    }

    fn diff_since(&self, since: Checksum) -> Result<WorldDiff, DiffError> {
        if since == self.checksum {
            return Ok(WorldDiff {
                tick: self.tick,
                base: since,
                checksum: self.checksum,
                added: Vec::new(),
                removed: Vec::new(),
                changed: Vec::new(),
            });
        }

        let start_index = self
            .history
            .iter()
            .position(|entry| entry.base == since)
            .ok_or_else(|| {
                if self.history.iter().any(|entry| entry.checksum == since) {
                    DiffError::ChecksumTooOld
                } else {
                    DiffError::UnknownChecksum
                }
            })?;

        let mut added: HashMap<u64, EntitySnapshot> = HashMap::new();
        let mut changed: HashMap<u64, EntitySnapshot> = HashMap::new();
        let mut removed: HashSet<u64> = HashSet::new();
        let mut current_checksum = since;
        let mut latest_checksum = since;

        for entry in self.history.iter().skip(start_index) {
            if entry.base != current_checksum {
                return Err(DiffError::ChecksumTooOld);
            }

            current_checksum = entry.checksum;
            latest_checksum = entry.checksum;

            for snapshot in &entry.added {
                removed.remove(&snapshot.id);
                changed.remove(&snapshot.id);
                added.insert(snapshot.id, snapshot.clone());
            }

            for snapshot in &entry.changed {
                if added.contains_key(&snapshot.id) {
                    added.insert(snapshot.id, snapshot.clone());
                } else if !removed.contains(&snapshot.id) {
                    changed.insert(snapshot.id, snapshot.clone());
                }
            }

            for id in &entry.removed {
                added.remove(id);
                changed.remove(id);
                removed.insert(*id);
            }
        }

        if latest_checksum == since {
            return Err(DiffError::ChecksumTooOld);
        }

        let mut added: Vec<_> = added.into_values().collect();
        added.sort_by_key(|e| e.id);
        let mut changed: Vec<_> = changed.into_values().collect();
        changed.sort_by_key(|e| e.id);
        let mut removed: Vec<_> = removed.into_iter().collect();
        removed.sort_unstable();

        Ok(WorldDiff {
            tick: self.tick,
            base: since,
            checksum: latest_checksum,
            added,
            removed,
            changed,
        })
    }
}

async fn get_world_snapshot(State(state): State<ServerState>) -> impl IntoResponse {
    let store = state.world.read().expect("world state lock");
    Json(store.snapshot())
}

async fn get_world_diff(
    State(state): State<ServerState>,
    Query(query): Query<WorldDiffQuery>,
) -> impl IntoResponse {
    let since_str = match query.since {
        Some(value) => value,
        None => return diff_error_response(DiffError::MissingSince),
    };

    let checksum = match since_str.parse::<Checksum>() {
        Ok(value) => value,
        Err(_) => return diff_error_response(DiffError::ParseError),
    };

    let store = state.world.read().expect("world state lock");
    match store.diff_since(checksum) {
        Ok(diff) => Json(diff).into_response(),
        Err(err) => diff_error_response(err),
    }
}

fn diff_error_response(err: DiffError) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: err.message().to_string(),
        }),
    )
        .into_response()
}

async fn post_intent(
    State(state): State<ServerState>,
    Json(payload): Json<IntentPayload>,
) -> impl IntoResponse {
    match validate_intent(payload) {
        Ok(intent) => match state.intents.send(intent) {
            Ok(()) => (
                StatusCode::ACCEPTED,
                Json(AcceptedResponse { status: "accepted" }),
            )
                .into_response(),
            Err(err) => {
                error!("failed to send intent: {err}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "intent channel closed".to_string(),
                    }),
                )
                    .into_response()
            }
        },
        Err(message) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: message.to_string(),
            }),
        )
            .into_response(),
    }
}

fn validate_intent(payload: IntentPayload) -> Result<Intent, &'static str> {
    let args = match payload.verb.as_str() {
        INTENT_SPAWN => sanitize_args::<SpawnArgs>(&payload.args)?,
        INTENT_MOVE => sanitize_args::<MoveArgs>(&payload.args)?,
        INTENT_APPLY_FORCE => sanitize_args::<ApplyForceArgs>(&payload.args)?,
        INTENT_DESPAWN => sanitize_args::<DespawnArgs>(&payload.args)?,
        _ => return Err("unknown intent verb"),
    };

    Ok(Intent {
        verb: payload.verb,
        args,
    })
}

fn sanitize_args<T>(value: &Value) -> Result<Value, &'static str>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    serde_json::from_value::<T>(value.clone())
        .map_err(|_| "invalid arguments")
        .and_then(|parsed| serde_json::to_value(parsed).map_err(|_| "invalid arguments"))
}
