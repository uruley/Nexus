//! Core primitives for the Nexus workspace.

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

