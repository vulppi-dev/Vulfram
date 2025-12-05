use glam::Vec2;
use std::collections::HashMap;

use crate::core::cmd::events::ModifiersState;

/// Cached keyboard state
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KeyboardStateCache {
    pub modifiers: ModifiersState,
}

impl KeyboardStateCache {
    pub fn new() -> Self {
        Self {
            modifiers: ModifiersState::default(),
        }
    }
}

/// Cached pointer (mouse) state per window
#[derive(Debug, Clone)]
pub struct PointerStateCache {
    pub position: Vec2,
}

impl PointerStateCache {
    pub fn new() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
        }
    }

    /// Check if position changed (with 1px threshold)
    pub fn position_changed(&self, new_pos: Vec2) -> bool {
        (self.position[0] - new_pos[0]).abs() > 1.0 || (self.position[1] - new_pos[1]).abs() > 1.0
    }
}

/// Manager for input state caches
#[derive(Debug, Default)]
pub struct InputCacheManager {
    pub keyboard: KeyboardStateCache,
    pub pointers: HashMap<u32, PointerStateCache>, // per window
}

impl InputCacheManager {
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardStateCache::new(),
            pointers: HashMap::new(),
        }
    }

    /// Get or create pointer cache for a window
    pub fn get_or_create_pointer(&mut self, window_id: u32) -> &mut PointerStateCache {
        self.pointers
            .entry(window_id)
            .or_insert_with(PointerStateCache::new)
    }

    /// Remove pointer cache for a window
    pub fn remove_pointer(&mut self, window_id: u32) {
        self.pointers.remove(&window_id);
    }
}
