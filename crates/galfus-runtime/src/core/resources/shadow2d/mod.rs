use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Shadow2dConfig {
    pub angular_resolution: u32,
    pub max_shadow_updates_per_frame: u32,
}

impl Default for Shadow2dConfig {
    fn default() -> Self {
        Self {
            angular_resolution: 1024,
            max_shadow_updates_per_frame: 16,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShadowManager2d {
    pub config: Shadow2dConfig,
    pub is_dirty: bool,
    pub last_scene_hash: Option<u64>,
    pub camera_light_hashes: HashMap<u32, Vec<u64>>,
    pub last_updated_layers: u32,
    pub update_cursor: usize,
}

impl ShadowManager2d {
    pub fn new() -> Self {
        Self {
            config: Shadow2dConfig::default(),
            is_dirty: true,
            last_scene_hash: None,
            camera_light_hashes: HashMap::new(),
            last_updated_layers: 0,
            update_cursor: 0,
        }
    }

    pub fn begin_frame(&mut self, _frame_index: u64) {
        // Keep 2D shadows fully real-time: always refresh light layers each frame.
        self.is_dirty = true;
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }

    pub fn should_update(&self, scene_hash: u64) -> bool {
        self.is_dirty || self.last_scene_hash != Some(scene_hash)
    }

    pub fn mark_updated(&mut self, scene_hash: u64) {
        self.last_scene_hash = Some(scene_hash);
        self.is_dirty = false;
    }

    pub fn should_update_light(&self, camera_id: u32, light_index: usize, layer_hash: u64) -> bool {
        if self.is_dirty {
            return true;
        }
        let Some(hashes) = self.camera_light_hashes.get(&camera_id) else {
            return true;
        };
        hashes.get(light_index).copied() != Some(layer_hash)
    }

    pub fn set_camera_light_hashes(&mut self, camera_id: u32, light_hashes: Vec<u64>) {
        self.camera_light_hashes.insert(camera_id, light_hashes);
    }
}
