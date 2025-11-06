use anchor::FrameTimings;
use bevy::prelude::*;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, update_hud);
    }
}

#[derive(Component)]
struct HudText;

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Frame: -- ms\nAnchor: -- ms\nRender: -- ms",
                TextStyle {
                    font,
                    font_size: 16.0,
                    color: Color::WHITE,
                },
            )
            .with_alignment(TextAlignment::Left),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(8.0),
                ..default()
            },
            ..default()
        },
        HudText,
    ));
}

fn update_hud(mut query: Query<&mut Text, With<HudText>>, timings: Res<FrameTimings>) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value = format!(
            "Frame: {:>5.2} ms\nAnchor: {:>5.2} ms\nRender: {:>5.2} ms",
            timings.frame_ms, timings.anchor_ms, timings.render_ms
        );

        let color = if timings.frame_ms > 16.0 {
            Color::YELLOW
        } else {
            Color::WHITE
        };
        text.sections[0].style.color = color;
    }
}
