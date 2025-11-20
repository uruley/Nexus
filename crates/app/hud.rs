use bevy::prelude::*;

/// Temporary no-op HUD plugin to avoid Bevy UI API mismatches while the renderer is validated.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, _app: &mut App) {
        // Intentionally empty for now.
    }
}
