//! World state crate placeholder.

use glam::Vec3;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityState {
    pub id: u64,
    pub position: Vec3,
}

pub fn log_entity(entity: &EntityState) {
    debug!(id = entity.id, position = ?entity.position, "Tracking entity state");
}
