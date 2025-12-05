use std::collections::HashMap;

use crate::core::cmd::events::ElementState;

/// Dead zone threshold for analog sticks and triggers
pub const GAMEPAD_AXIS_DEAD_ZONE: f32 = 0.1;

/// Minimum change threshold to dispatch axis events
pub const GAMEPAD_AXIS_CHANGE_THRESHOLD: f32 = 0.01;

/// Minimum change threshold to dispatch button analog value events
pub const GAMEPAD_BUTTON_CHANGE_THRESHOLD: f32 = 0.05;

/// Cached state for a single gamepad
#[derive(Debug, Clone)]
pub struct GamepadStateCache {
    #[allow(dead_code)]
    pub connected: bool,
    #[allow(dead_code)]
    pub name: String,
    pub axes: HashMap<u32, f32>,
    pub buttons: HashMap<u32, (ElementState, f32)>,
}

impl GamepadStateCache {
    pub fn new(name: String) -> Self {
        Self {
            connected: true,
            name,
            axes: HashMap::new(),
            buttons: HashMap::new(),
        }
    }

    /// Apply dead zone to axis value
    pub fn apply_dead_zone(value: f32) -> f32 {
        if value.abs() < GAMEPAD_AXIS_DEAD_ZONE {
            0.0
        } else {
            // Rescale value to maintain smooth transition
            let sign = value.signum();
            let adjusted = (value.abs() - GAMEPAD_AXIS_DEAD_ZONE) / (1.0 - GAMEPAD_AXIS_DEAD_ZONE);
            sign * adjusted
        }
    }

    /// Check if axis value changed significantly
    pub fn axis_changed(&self, axis: u32, new_value: f32) -> bool {
        let adjusted_value = Self::apply_dead_zone(new_value);

        if let Some(&cached_value) = self.axes.get(&axis) {
            (cached_value - adjusted_value).abs() > GAMEPAD_AXIS_CHANGE_THRESHOLD
        } else {
            adjusted_value.abs() > GAMEPAD_AXIS_CHANGE_THRESHOLD
        }
    }

    /// Check if button state or value changed significantly
    pub fn button_changed(&self, button: u32, new_state: ElementState, new_value: f32) -> bool {
        if let Some(&(cached_state, cached_value)) = self.buttons.get(&button) {
            // State changed OR value changed significantly
            cached_state != new_state
                || (cached_value - new_value).abs() > GAMEPAD_BUTTON_CHANGE_THRESHOLD
        } else {
            // First time seeing this button
            true
        }
    }

    /// Get cached axis value with dead zone applied
    pub fn get_axis_value(&self, axis: u32) -> f32 {
        self.axes.get(&axis).copied().unwrap_or(0.0)
    }

    /// Update axis cache
    pub fn update_axis(&mut self, axis: u32, value: f32) {
        let adjusted_value = Self::apply_dead_zone(value);
        self.axes.insert(axis, adjusted_value);
    }

    /// Update button cache
    pub fn update_button(&mut self, button: u32, state: ElementState, value: f32) {
        self.buttons.insert(button, (state, value));
    }
}

/// Manager for all gamepad state caches
#[derive(Debug, Default)]
pub struct GamepadCacheManager {
    pub gamepads: HashMap<u32, GamepadStateCache>,
}

impl GamepadCacheManager {
    pub fn new() -> Self {
        Self {
            gamepads: HashMap::new(),
        }
    }

    /// Create cache for a newly connected gamepad
    pub fn add_gamepad(&mut self, gamepad_id: u32, name: String) {
        self.gamepads
            .insert(gamepad_id, GamepadStateCache::new(name));
    }

    /// Remove cache for a disconnected gamepad
    pub fn remove_gamepad(&mut self, gamepad_id: u32) {
        self.gamepads.remove(&gamepad_id);
    }

    /// Get cache for a gamepad
    #[allow(dead_code)]
    pub fn get(&self, gamepad_id: u32) -> Option<&GamepadStateCache> {
        self.gamepads.get(&gamepad_id)
    }

    /// Get mutable cache for a gamepad
    pub fn get_mut(&mut self, gamepad_id: u32) -> Option<&mut GamepadStateCache> {
        self.gamepads.get_mut(&gamepad_id)
    }
}
