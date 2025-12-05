use serde::{Deserialize, Serialize};

use super::common::ElementState;

/// Gamepad events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum GamepadEvent {
    /// Gamepad was connected
    #[serde(rename_all = "camelCase")]
    OnConnect { gamepad_id: u32, name: String },

    /// Gamepad was disconnected
    #[serde(rename_all = "camelCase")]
    OnDisconnect { gamepad_id: u32 },

    /// Gamepad button pressed/released
    #[serde(rename_all = "camelCase")]
    OnButton {
        gamepad_id: u32,
        button: u32,
        state: ElementState,
        value: f32, // 0.0-1.0 for analog triggers
    },

    /// Gamepad axis moved
    #[serde(rename_all = "camelCase")]
    OnAxis {
        gamepad_id: u32,
        axis: u32,
        value: f32, // -1.0 to 1.0 for sticks, 0.0 to 1.0 for triggers
    },
}
