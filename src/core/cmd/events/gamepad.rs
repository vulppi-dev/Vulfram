use serde::{Deserialize, Serialize};

use super::common::ElementState;

/// Gamepad button types following standard gamepad mapping
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GamepadButton {
    // Face buttons
    South = 0, // A / Cross
    East = 1,  // B / Circle
    West = 2,  // X / Square
    North = 3, // Y / Triangle

    // Shoulder buttons
    LeftBumper = 4,
    RightBumper = 5,
    LeftTrigger = 6,
    RightTrigger = 7,

    // Center buttons
    Select = 8,
    Start = 9,
    Mode = 10, // Guide / Home

    // Stick buttons
    LeftStick = 11,
    RightStick = 12,

    // D-pad
    DpadUp = 13,
    DpadDown = 14,
    DpadLeft = 15,
    DpadRight = 16,

    // Other
    Other(u8),
}

/// Gamepad axis types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GamepadAxis {
    LeftStickX = 0,
    LeftStickY = 1,
    RightStickX = 2,
    RightStickY = 3,
    LeftTrigger = 4,
    RightTrigger = 5,
    Other(u8),
}

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
        button: GamepadButton,
        state: ElementState,
        value: f32, // 0.0-1.0 for analog triggers
    },

    /// Gamepad axis moved
    #[serde(rename_all = "camelCase")]
    OnAxis {
        gamepad_id: u32,
        axis: GamepadAxis,
        value: f32, // -1.0 to 1.0 for sticks, 0.0 to 1.0 for triggers
    },
}
