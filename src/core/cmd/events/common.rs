use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Represents the state of an input element (pressed or released)
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
pub enum ElementState {
    Released = 0,
    Pressed = 1,
}

/// Represents the phase of a touch/gesture event
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
pub enum TouchPhase {
    Started = 0,
    Moved = 1,
    Ended = 2,
    Cancelled = 3,
}

/// Represents keyboard modifier keys state
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifiersState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}
