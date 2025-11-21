use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Describes a raw motion input that can be compiled into a reusable animation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MotionSource {
    VideoClip { id: String, path: String },
    KeypointsSequence { id: String },
    PoseStream { id: String },
    Synthetic { id: String, description: String },
}

impl MotionSource {
    /// Returns the identifier associated with the motion source.
    pub fn id(&self) -> &str {
        match self {
            MotionSource::VideoClip { id, .. }
            | MotionSource::KeypointsSequence { id }
            | MotionSource::PoseStream { id }
            | MotionSource::Synthetic { id, .. } => id,
        }
    }
}

/// Identifies a target rig or skeleton configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RigTarget {
    pub id: String,
    pub bone_count: Option<u32>,
    pub naming_scheme: Option<String>,
}

/// Represents compiled motion data ready for the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledMotion {
    pub id: String,
    pub rig_target: RigTarget,
    pub frame_count: u32,
    pub fps: f32,
    pub has_root_motion: bool,
    pub data: Vec<u8>,
}

/// Errors surfaced by the motion compiler subsystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MotionError {
    InvalidInput(String),
    BackendUnavailable,
    Internal(String),
}

impl Display for MotionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MotionError::InvalidInput(msg) => write!(f, "Invalid motion input: {msg}"),
            MotionError::BackendUnavailable => write!(f, "Motion compiler backend unavailable"),
            MotionError::Internal(msg) => write!(f, "Internal motion compiler error: {msg}"),
        }
    }
}

impl Error for MotionError {}

/// Convenience result alias for the motion compiler.
pub type MotionResult<T> = Result<T, MotionError>;

/// Trait for pluggable motion compiler backends.
pub trait MotionCompilerBackend {
    fn name(&self) -> &str;
    fn compile(&self, source: &MotionSource, target: &RigTarget) -> MotionResult<CompiledMotion>;
}

/// Trait for storing and retrieving compiled motions.
pub trait MotionLibrary {
    fn store(&mut self, motion: CompiledMotion) -> MotionResult<()>;
    fn get(&self, id: &str) -> Option<&CompiledMotion>;
}

/// A simple mock backend that fabricates compiled motions.
#[derive(Debug, Clone)]
pub struct MockMotionCompiler {
    name: String,
}

impl MockMotionCompiler {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Default for MockMotionCompiler {
    fn default() -> Self {
        Self::new("mock")
    }
}

impl MotionCompilerBackend for MockMotionCompiler {
    fn name(&self) -> &str {
        &self.name
    }

    fn compile(&self, source: &MotionSource, target: &RigTarget) -> MotionResult<CompiledMotion> {
        let id = format!("{}::{}", source.id(), target.id);
        Ok(CompiledMotion {
            id,
            rig_target: target.clone(),
            frame_count: 30,
            fps: 30.0,
            has_root_motion: false,
            data: Vec::new(),
        })
    }
}

/// In-memory motion library backed by a HashMap.
#[derive(Debug, Default)]
pub struct InMemoryMotionLibrary {
    motions: HashMap<String, CompiledMotion>,
}

impl InMemoryMotionLibrary {
    pub fn new() -> Self {
        Self {
            motions: HashMap::new(),
        }
    }
}

impl MotionLibrary for InMemoryMotionLibrary {
    fn store(&mut self, motion: CompiledMotion) -> MotionResult<()> {
        self.motions.insert(motion.id.clone(), motion);
        Ok(())
    }

    fn get(&self, id: &str) -> Option<&CompiledMotion> {
        self.motions.get(id)
    }
}

/// Basic configuration for selecting and configuring a motion compiler backend.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MotionCompilerConfig {
    pub backend: String,
    pub settings: HashMap<String, String>,
}

/// Factory helper to build a motion compiler backend from configuration.
pub fn build_motion_compiler_from_config(
    cfg: &MotionCompilerConfig,
) -> MotionResult<Box<dyn MotionCompilerBackend>> {
    match cfg.backend.as_str() {
        "mock" | "" => Ok(Box::new(MockMotionCompiler::new("mock"))),
        other => Err(MotionError::InvalidInput(format!(
            "Unknown motion compiler backend: {other}"
        ))),
    }
}
