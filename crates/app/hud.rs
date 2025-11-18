use anchor::FrameTimings;
use bevy::prelude::*;

use crate::avatar::AnimateAvatar;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud).add_systems(
            Update,
            (update_hud, handle_toggle_button, update_toggle_label),
        );
    }
}

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct AnimateToggleButton;

#[derive(Component)]
struct AnimateToggleLabel;

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Frame: -- ms\nAnchor: -- ms\nRender: -- ms",
                TextStyle {
                    font: font.clone(),
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

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(8.0),
                    right: Val::Px(8.0),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                    ..default()
                },
                background_color: Color::srgb(0.18, 0.18, 0.22).into(),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            AnimateToggleButton,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Animate Avatar From Pose: On",
                    TextStyle {
                        font,
                        font_size: 16.0,
                        color: Color::WHITE,
                    },
                )
                .with_text_alignment(TextAlignment::Center),
                AnimateToggleLabel,
            ));
        });
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

fn handle_toggle_button(
    mut animate: Option<ResMut<AnimateAvatar>>,
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<AnimateToggleButton>),
    >,
) {
    let Some(mut animate) = animate else {
        return;
    };

    for (interaction, mut color) in &mut buttons {
        match *interaction {
            Interaction::Pressed => {
                animate.0 = !animate.0;
                *color = Color::srgb(0.30, 0.30, 0.35).into();
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.24, 0.24, 0.28).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.18, 0.18, 0.22).into();
            }
        }
    }
}

fn update_toggle_label(
    animate: Option<Res<AnimateAvatar>>,
    mut query: Query<&mut Text, With<AnimateToggleLabel>>,
) {
    let Some(animate) = animate else {
        return;
    };

    if !animate.is_changed() {
        return;
    }

    if let Ok(mut text) = query.get_single_mut() {
        let state = if animate.0 { "On" } else { "Off" };
        text.sections[0].value = format!("Animate Avatar From Pose: {state}");
    }
}
