use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::render::{Render, RenderApp, RenderSet};
use tracing::info;

#[derive(Resource, Debug, Clone)]
pub struct FrameTimings {
    pub frame_ms: f32,
    pub anchor_ms: f32,
    pub render_ms: f32,
}

impl Default for FrameTimings {
    fn default() -> Self {
        Self {
            frame_ms: 0.0,
            anchor_ms: 0.0,
            render_ms: 0.0,
        }
    }
}

#[derive(Resource)]
struct MetricsState {
    frame_start: Instant,
    frame_initialized: bool,
    anchor_start: Option<Instant>,
    accumulated_anchor: Duration,
    last_log: Instant,
    frames_since_log: u32,
    anchor_steps_since_log: u32,
    total_tick: u64,
    frame_duration_since_log: Duration,
    anchor_duration_since_log: Duration,
    render_duration_since_log: Duration,
}

impl Default for MetricsState {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            frame_start: now,
            frame_initialized: false,
            anchor_start: None,
            accumulated_anchor: Duration::ZERO,
            last_log: now,
            frames_since_log: 0,
            anchor_steps_since_log: 0,
            total_tick: 0,
            frame_duration_since_log: Duration::ZERO,
            anchor_duration_since_log: Duration::ZERO,
            render_duration_since_log: Duration::ZERO,
        }
    }
}

#[derive(Default)]
struct RenderTimingShared {
    start: Option<Instant>,
    end: Option<Instant>,
}

#[derive(Resource, Clone, Default)]
struct SharedRenderTiming(Arc<Mutex<RenderTimingShared>>);

pub(crate) fn init_metrics(app: &mut App) {
    let shared = SharedRenderTiming::default();

    app.insert_resource(FrameTimings::default())
        .insert_resource(MetricsState::default())
        .insert_resource(shared.clone())
        .add_systems(First, begin_frame)
        .add_systems(Last, mark_render_start)
        .add_systems(FixedFirst, anchor_step_start)
        .add_systems(FixedLast, anchor_step_end);

    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.insert_resource(shared);
        render_app.add_systems(Render, finish_render_timer.in_set(RenderSet::Cleanup));
    }
}

fn begin_frame(
    mut state: ResMut<MetricsState>,
    mut timings: ResMut<FrameTimings>,
    shared: Res<SharedRenderTiming>,
) {
    let now = Instant::now();
    let frame_duration = now - state.frame_start;
    let anchor_duration = state.accumulated_anchor;

    let mut render_duration = Duration::ZERO;
    if let Ok(mut data) = shared.0.lock() {
        if let (Some(start), Some(end)) = (data.start.take(), data.end.take()) {
            render_duration = end.saturating_duration_since(start);
        }
    }

    if state.frame_initialized {
        timings.frame_ms = frame_duration.as_secs_f32() * 1000.0;
        timings.anchor_ms = anchor_duration.as_secs_f32() * 1000.0;
        timings.render_ms = render_duration.as_secs_f32() * 1000.0;

        state.frames_since_log += 1;
        state.frame_duration_since_log += frame_duration;
        state.anchor_duration_since_log += anchor_duration;
        state.render_duration_since_log += render_duration;

        if now.duration_since(state.last_log) >= Duration::from_secs(1) {
            let frames = state.frames_since_log.max(1) as f64;
            info!(
                tick = state.total_tick,
                frames = state.frames_since_log,
                anchor_steps = state.anchor_steps_since_log,
                avg_frame_ms = state.frame_duration_since_log.as_secs_f64() * 1000.0 / frames,
                avg_anchor_ms = state.anchor_duration_since_log.as_secs_f64() * 1000.0 / frames,
                avg_render_ms = state.render_duration_since_log.as_secs_f64() * 1000.0 / frames,
                "frame timings"
            );

            state.last_log = now;
            state.frames_since_log = 0;
            state.anchor_steps_since_log = 0;
            state.frame_duration_since_log = Duration::ZERO;
            state.anchor_duration_since_log = Duration::ZERO;
            state.render_duration_since_log = Duration::ZERO;
        }
    } else {
        state.frame_initialized = true;
        timings.frame_ms = 0.0;
        timings.anchor_ms = 0.0;
        timings.render_ms = 0.0;
    }

    state.frame_start = now;
    state.accumulated_anchor = Duration::ZERO;
}

fn mark_render_start(shared: Res<SharedRenderTiming>) {
    if let Ok(mut data) = shared.0.lock() {
        data.start = Some(Instant::now());
        data.end = None;
    }
}

fn finish_render_timer(shared: Res<SharedRenderTiming>) {
    if let Ok(mut data) = shared.0.lock() {
        data.end = Some(Instant::now());
    }
}

fn anchor_step_start(mut state: ResMut<MetricsState>) {
    state.anchor_start = Some(Instant::now());
}

fn anchor_step_end(mut state: ResMut<MetricsState>) {
    if let Some(start) = state.anchor_start.take() {
        let now = Instant::now();
        state.accumulated_anchor += now.saturating_duration_since(start);
        state.anchor_steps_since_log += 1;
        state.total_tick += 1;
    }
}
