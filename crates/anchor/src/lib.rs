//! Anchor crate placeholder.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorConfig {
    pub name: String,
}

impl AnchorConfig {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn initialize(&self) -> Result<()> {
        info!(anchor = %self.name, "Initializing anchor");
        Ok(())
    }
}
