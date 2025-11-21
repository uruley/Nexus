//! A lightweight motion compiler stub that converts pose frames into motion clips.
//!
//! This crate provides the initial API surface for turning raw pose data into
//! animation clips. The implementation is intentionally simple for now so that
//! other parts of the engine can start integrating against the API while we
//! iterate on real machine learning backends.

/// Represents a single pose sample coming from perception.
#[derive(Debug, Clone, PartialEq)]
pub struct PoseFrame {
    /// Timestamp of the pose sample in seconds.
    pub time: f32,
    /// Named joint positions captured for this frame. Each joint is represented
    /// by a name and a 3D position vector.
    pub joints: Vec<(String, [f32; 3])>,
}

/// Represents a frame in a compiled motion clip.
#[derive(Debug, Clone, PartialEq)]
pub struct MotionFrame {
    pub time: f32,
    pub joints: Vec<(String, [f32; 3])>,
}

/// A collection of motion frames that form an animation clip.
#[derive(Debug, Clone, PartialEq)]
pub struct MotionClip {
    pub name: String,
    pub frames: Vec<MotionFrame>,
}

/// Compile a sequence of pose frames into a motion clip.
///
/// The compilation is currently a direct translation of pose data into
/// `MotionFrame` instances. Future iterations will enrich this function with
/// retargeting, filtering, and compression steps.
pub fn compile_from_pose_sequence(name: &str, poses: &[PoseFrame]) -> MotionClip {
    let frames = poses
        .iter()
        .map(|pose| MotionFrame {
            time: pose.time,
            joints: pose.joints.clone(),
        })
        .collect();

    MotionClip {
        name: name.to_string(),
        frames,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_pose_frame(time: f32, joint_offset: f32) -> PoseFrame {
        PoseFrame {
            time,
            joints: vec![
                ("root".to_string(), [joint_offset, joint_offset + 1.0, 0.0]),
                ("hand".to_string(), [joint_offset + 0.5, joint_offset + 1.5, 0.2]),
            ],
        }
    }

    #[test]
    fn compiles_pose_sequence_into_motion_clip() {
        let poses = vec![mock_pose_frame(0.0, 0.0), mock_pose_frame(0.033, 1.0)];

        let clip = compile_from_pose_sequence("wave", &poses);

        assert_eq!(clip.name, "wave");
        assert_eq!(clip.frames.len(), poses.len());

        assert_eq!(clip.frames[0].time, poses[0].time);
        assert_eq!(clip.frames[0].joints, poses[0].joints);
    }

    #[test]
    fn handles_empty_pose_sequence() {
        let poses: Vec<PoseFrame> = Vec::new();

        let clip = compile_from_pose_sequence("idle", &poses);

        assert_eq!(clip.name, "idle");
        assert!(clip.frames.is_empty());
    }
}
