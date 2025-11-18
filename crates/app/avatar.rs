use bevy::prelude::*;

use crate::perception::PerceptionFrameLatest;

const MOVENET_KEYPOINT_COUNT: usize = 17;
const CONFIDENCE_THRESHOLD: f32 = 0.25;
const TARGET_HEIGHT: f32 = 2.0;
const BONE_RADIUS_SCALE: f32 = 0.1;

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimateAvatar>()
            .add_systems(Startup, spawn_avatar)
            .add_systems(Update, (update_pose_from_frame, apply_pose_to_rig));
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct AnimateAvatar(pub bool);

impl Default for AnimateAvatar {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct PoseSample {
    pub position: Vec2,
    pub confidence: f32,
}

#[derive(Component, Debug)]
pub struct Pose2D {
    pub width: f32,
    pub height: f32,
    pub keypoints: [Option<PoseSample>; MOVENET_KEYPOINT_COUNT],
}

impl Default for Pose2D {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            keypoints: [None; MOVENET_KEYPOINT_COUNT],
        }
    }
}

impl Pose2D {
    fn clear(&mut self) {
        self.keypoints = [None; MOVENET_KEYPOINT_COUNT];
    }
}

#[derive(Component)]
pub struct PoseApplier {
    alpha: f32,
    smoothed: [Option<Vec2>; MOVENET_KEYPOINT_COUNT],
    confidence: [f32; MOVENET_KEYPOINT_COUNT],
}

impl PoseApplier {
    pub fn new(alpha: f32) -> Self {
        Self {
            alpha,
            smoothed: [None; MOVENET_KEYPOINT_COUNT],
            confidence: [0.0; MOVENET_KEYPOINT_COUNT],
        }
    }
}

#[derive(Component)]
struct PerceptionAvatar;

#[derive(Component)]
struct PoseBone {
    start: usize,
    end: usize,
}

struct BoneDefinition {
    name: &'static str,
    start: &'static str,
    end: &'static str,
}

const BONE_DEFINITIONS: [BoneDefinition; 15] = [
    BoneDefinition {
        name: "bone_nose_left_eye",
        start: "nose",
        end: "left_eye",
    },
    BoneDefinition {
        name: "bone_nose_right_eye",
        start: "nose",
        end: "right_eye",
    },
    BoneDefinition {
        name: "bone_left_right_eye",
        start: "left_eye",
        end: "right_eye",
    },
    BoneDefinition {
        name: "bone_shoulders",
        start: "left_shoulder",
        end: "right_shoulder",
    },
    BoneDefinition {
        name: "bone_left_upper_arm",
        start: "left_shoulder",
        end: "left_elbow",
    },
    BoneDefinition {
        name: "bone_left_lower_arm",
        start: "left_elbow",
        end: "left_wrist",
    },
    BoneDefinition {
        name: "bone_right_upper_arm",
        start: "right_shoulder",
        end: "right_elbow",
    },
    BoneDefinition {
        name: "bone_right_lower_arm",
        start: "right_elbow",
        end: "right_wrist",
    },
    BoneDefinition {
        name: "bone_left_upper_body",
        start: "left_shoulder",
        end: "left_hip",
    },
    BoneDefinition {
        name: "bone_right_upper_body",
        start: "right_shoulder",
        end: "right_hip",
    },
    BoneDefinition {
        name: "bone_hips",
        start: "left_hip",
        end: "right_hip",
    },
    BoneDefinition {
        name: "bone_left_upper_leg",
        start: "left_hip",
        end: "left_knee",
    },
    BoneDefinition {
        name: "bone_left_lower_leg",
        start: "left_knee",
        end: "left_ankle",
    },
    BoneDefinition {
        name: "bone_right_upper_leg",
        start: "right_hip",
        end: "right_knee",
    },
    BoneDefinition {
        name: "bone_right_lower_leg",
        start: "right_knee",
        end: "right_ankle",
    },
];

fn spawn_avatar(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Cylinder {
        radius: 0.5,
        height: 1.0,
        resolution: 12,
        segments: 1,
    }));

    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.3, 0.4),
        perceptual_roughness: 0.4,
        metallic: 0.0,
        ..default()
    });

    let mut entity_commands = commands.spawn((
        Name::new("PerceptionAvatar_p0"),
        SpatialBundle::from_transform(Transform::from_translation(Vec3::new(0.0, 1.0, 0.0))),
        Pose2D::default(),
        PoseApplier::new(0.6),
        PerceptionAvatar,
    ));

    entity_commands.with_children(|parent| {
        for def in BONE_DEFINITIONS {
            let Some(start) = keypoint_index(def.start) else {
                continue;
            };
            let Some(end) = keypoint_index(def.end) else {
                continue;
            };
            parent.spawn((
                PbrBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    transform: Transform::default(),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                PoseBone { start, end },
                Name::new(def.name),
            ));
        }
    });
}

fn update_pose_from_frame(
    latest: Res<PerceptionFrameLatest>,
    mut query: Query<&mut Pose2D, With<PerceptionAvatar>>,
) {
    let Ok(mut pose) = query.get_single_mut() else {
        return;
    };

    pose.clear();

    let Some(frame) = latest.0.as_ref() else {
        return;
    };

    pose.width = frame.size[0] as f32;
    pose.height = frame.size[1] as f32;

    let person = frame
        .persons
        .iter()
        .find(|person| person.id.as_deref() == Some("p0"))
        .or_else(|| frame.persons.first());

    let Some(person) = person else {
        return;
    };

    for keypoint in &person.keypoints {
        if let Some(index) = keypoint_index(&keypoint.name) {
            pose.keypoints[index] = Some(PoseSample {
                position: Vec2::new(keypoint.x, keypoint.y),
                confidence: keypoint.c,
            });
        }
    }
}

fn apply_pose_to_rig(
    animate: Res<AnimateAvatar>,
    mut rig_query: Query<(&Pose2D, &mut PoseApplier, &Children), With<PerceptionAvatar>>,
    mut bone_query: Query<(&PoseBone, &mut Transform, &mut Visibility)>,
) {
    let Ok((pose, mut applier, children)) = rig_query.get_single_mut() else {
        return;
    };

    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    let mut has_valid = false;

    for sample in pose.keypoints.iter().flatten() {
        if sample.confidence < CONFIDENCE_THRESHOLD {
            continue;
        }
        min = min.min(sample.position);
        max = max.max(sample.position);
        has_valid = true;
    }

    if !has_valid {
        if animate.0 {
            for child in children.iter() {
                if let Ok((_bone, _transform, mut visibility)) = bone_query.get_mut(*child) {
                    *visibility = Visibility::Hidden;
                }
            }
        }
        return;
    }

    let center = (min + max) * 0.5;
    let height = (max.y - min.y).max(1.0);
    let scale = TARGET_HEIGHT / height;

    let mut positions = [Option::<Vec3>::None; MOVENET_KEYPOINT_COUNT];

    for (index, sample_opt) in pose.keypoints.iter().enumerate() {
        let Some(sample) = sample_opt else {
            applier.confidence[index] = 0.0;
            continue;
        };

        if sample.confidence < CONFIDENCE_THRESHOLD {
            applier.confidence[index] = 0.0;
            continue;
        }

        let normalized = Vec2::new(
            (sample.position.x - center.x) * scale,
            (center.y - sample.position.y) * scale,
        );

        let smoothed = match applier.smoothed[index] {
            Some(previous) => previous.lerp(normalized, applier.alpha),
            None => normalized,
        };

        applier.smoothed[index] = Some(smoothed);
        applier.confidence[index] = sample.confidence;
        positions[index] = Some(smoothed.extend(0.0));
    }

    if !animate.0 {
        return;
    }

    for child in children.iter() {
        if let Ok((bone, mut transform, mut visibility)) = bone_query.get_mut(*child) {
            let Some(start) = positions[bone.start] else {
                *visibility = Visibility::Hidden;
                continue;
            };
            let Some(end) = positions[bone.end] else {
                *visibility = Visibility::Hidden;
                continue;
            };

            if applier.confidence[bone.start] < CONFIDENCE_THRESHOLD
                || applier.confidence[bone.end] < CONFIDENCE_THRESHOLD
            {
                *visibility = Visibility::Hidden;
                continue;
            }

            let direction = end - start;
            let length = direction.length();
            if length < f32::EPSILON {
                *visibility = Visibility::Hidden;
                continue;
            }

            let mid = (start + end) * 0.5;
            let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());

            transform.translation = mid;
            transform.rotation = rotation;
            transform.scale = Vec3::new(BONE_RADIUS_SCALE, length, BONE_RADIUS_SCALE);
            *visibility = Visibility::Visible;
        }
    }
}

fn keypoint_index(name: &str) -> Option<usize> {
    match name {
        "nose" => Some(0),
        "left_eye" => Some(1),
        "right_eye" => Some(2),
        "left_ear" => Some(3),
        "right_ear" => Some(4),
        "left_shoulder" => Some(5),
        "right_shoulder" => Some(6),
        "left_elbow" => Some(7),
        "right_elbow" => Some(8),
        "left_wrist" => Some(9),
        "right_wrist" => Some(10),
        "left_hip" => Some(11),
        "right_hip" => Some(12),
        "left_knee" => Some(13),
        "right_knee" => Some(14),
        "left_ankle" => Some(15),
        "right_ankle" => Some(16),
        _ => None,
    }
}
