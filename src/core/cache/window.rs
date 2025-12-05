use glam::{IVec2, UVec2};
use std::collections::HashMap;

/// Cached window state to detect actual changes
#[derive(Debug, Clone)]
pub struct WindowStateCache {
    pub inner_position: IVec2,
    pub outer_position: IVec2,
    pub inner_size: UVec2,
    pub outer_size: UVec2,
    pub scale_factor: f64,
    pub focused: bool,
    pub occluded: bool,
    pub dark_mode: bool,
}

impl WindowStateCache {
    pub fn new() -> Self {
        Self {
            inner_position: IVec2::new(0, 0),
            outer_position: IVec2::new(0, 0),
            inner_size: UVec2::new(0, 0),
            outer_size: UVec2::new(0, 0),
            scale_factor: 1.0,
            focused: false,
            occluded: false,
            dark_mode: false,
        }
    }

    /// Check if position changed (with 1px threshold)
    pub fn position_changed(&self, new_pos: IVec2) -> bool {
        (self.inner_position[0] - new_pos[0]).abs() > 0
            || (self.inner_position[1] - new_pos[1]).abs() > 0
    }

    /// Check if size changed (exact comparison needed)
    pub fn size_changed(&self, new_size: UVec2) -> bool {
        self.inner_size[0] != new_size[0] || self.inner_size[1] != new_size[1]
    }

    /// Check if scale factor changed (with epsilon threshold)
    pub fn scale_factor_changed(&self, new_scale: f64) -> bool {
        (self.scale_factor - new_scale).abs() > 0.0001
    }
}

/// Manager for all window state caches
#[derive(Debug, Default)]
pub struct WindowCacheManager {
    pub caches: HashMap<u32, WindowStateCache>,
}

impl WindowCacheManager {
    pub fn new() -> Self {
        Self {
            caches: HashMap::new(),
        }
    }

    /// Get or create cache for a window
    pub fn get_or_create(&mut self, window_id: u32) -> &mut WindowStateCache {
        self.caches
            .entry(window_id)
            .or_insert_with(WindowStateCache::new)
    }

    /// Remove cache for a window
    pub fn remove(&mut self, window_id: u32) {
        self.caches.remove(&window_id);
    }
}
