use serde::{Deserialize, Serialize};

use super::common::{ElementState, ModifiersState};

/// Keyboard input event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum KeyboardEvent {
    /// Key was pressed or released
    #[serde(rename_all = "camelCase")]
    OnInput {
        window_id: u32,
        key_code: u32,
        state: ElementState,
        location: u32,
        repeat: bool,
        text: Option<String>,
        modifiers: ModifiersState,
    },

    /// Modifiers changed
    #[serde(rename_all = "camelCase")]
    OnModifiersChange {
        window_id: u32,
        modifiers: ModifiersState,
    },

    /// IME composition started
    #[serde(rename_all = "camelCase")]
    OnImeEnable { window_id: u32 },

    /// IME composition in progress
    #[serde(rename_all = "camelCase")]
    OnImePreedit {
        window_id: u32,
        text: String,
        cursor_range: Option<(usize, usize)>,
    },

    /// IME composition committed
    #[serde(rename_all = "camelCase")]
    OnImeCommit { window_id: u32, text: String },

    /// IME disabled
    #[serde(rename_all = "camelCase")]
    OnImeDisable { window_id: u32 },
}
