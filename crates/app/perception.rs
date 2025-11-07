use bevy::prelude::*;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Resource)]
pub struct PerceptionConfig {
    pub endpoint: String, // e.g. http://127.0.0.1:5055/frame
}

impl Default for PerceptionConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:5055/frame".into(),
        }
    }
}

#[derive(Resource)]
pub struct PerceptionHttpClient(pub Client);

impl FromWorld for PerceptionHttpClient {
    fn from_world(_: &mut World) -> Self {
        Self(Client::new())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Keypoint {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub c: f32,
}

#[derive(Deserialize, Debug)]
pub struct Person {
    pub id: Option<String>,
    pub score: f32,
    pub bbox: [f32; 4],
    #[serde(default)]
    pub keypoints: Vec<Keypoint>,
}

#[derive(Deserialize, Debug)]
pub struct Depth {
    pub format: String,
    pub uri: String,
}

#[derive(Deserialize, Debug)]
pub struct PerceptionFrame {
    pub ts: u64,
    pub size: [u32; 2],
    pub depth: Option<Depth>,
    #[serde(default)]
    pub persons: Vec<Person>,
}

#[derive(Resource, Default)]
pub struct PerceptionFrameLatest(pub Option<PerceptionFrame>);

pub struct PerceptionBridgePlugin;

impl Plugin for PerceptionBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PerceptionConfig>()
            .insert_resource(PerceptionFrameLatest::default())
            .init_resource::<PerceptionHttpClient>()
            .add_systems(Update, poll_perception);
    }
}

fn poll_perception(
    mut latest: ResMut<PerceptionFrameLatest>,
    cfg: Res<PerceptionConfig>,
    client: Res<PerceptionHttpClient>,
    mut frame_counter: Local<u32>,
) {
    *frame_counter = (*frame_counter + 1) % 6;
    if *frame_counter != 0 {
        return;
    }

    if let Ok(resp) = client.0.get(&cfg.endpoint).send() {
        if let Ok(pf) = resp.json::<PerceptionFrame>() {
            latest.0 = Some(pf);
        }
    }
}
