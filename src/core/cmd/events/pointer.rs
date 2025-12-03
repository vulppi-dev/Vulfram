use serde::{Deserialize, Serialize};

use super::common::{ElementState, TouchPhase};
use crate::core::units::Vector2;

/// Mouse button types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MouseButton {
    Left = 0,
    Right = 1,
    Middle = 2,
    Back = 3,
    Forward = 4,
    Other(u8),
}

/// Pointer type for unified mouse/touch handling
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PointerType {
    Mouse = 0,
    Touch = 1,
    Pen = 2,
}

/// Mouse scroll delta type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "kebab-case")]
pub enum ScrollDelta {
    /// Line-based scrolling (traditional mouse wheel)
    Line(Vector2),
    /// Pixel-based scrolling (touchpad)
    Pixel(Vector2),
}

/// Pointer (Mouse/Touch) events - unified for both input types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum PointerEvent {
    /// Pointer moved
    #[serde(rename_all = "camelCase")]
    OnMove {
        window_id: u32,
        pointer_type: PointerType,
        pointer_id: u64,
        position: Vector2,
    },

    /// Pointer entered window area
    #[serde(rename_all = "camelCase")]
    OnEnter {
        window_id: u32,
        pointer_type: PointerType,
        pointer_id: u64,
    },

    /// Pointer left window area
    #[serde(rename_all = "camelCase")]
    OnLeave {
        window_id: u32,
        pointer_type: PointerType,
        pointer_id: u64,
    },

    /// Pointer button pressed/released (mouse) or touch started/ended
    #[serde(rename_all = "camelCase")]
    OnButton {
        window_id: u32,
        pointer_type: PointerType,
        pointer_id: u64,
        button: MouseButton,
        state: ElementState,
        position: Vector2,
    },

    /// Mouse wheel/touchpad scroll
    #[serde(rename_all = "camelCase")]
    OnScroll {
        window_id: u32,
        delta: ScrollDelta,
        phase: TouchPhase,
    },

    /// Touch event with pressure and additional info
    #[serde(rename_all = "camelCase")]
    OnTouch {
        window_id: u32,
        pointer_id: u64,
        phase: TouchPhase,
        position: Vector2,
        pressure: Option<f32>,
    },

    /// Pinch gesture (zoom)
    #[serde(rename_all = "camelCase")]
    OnPinchGesture {
        window_id: u32,
        delta: f64,
        phase: TouchPhase,
    },

    /// Pan gesture
    #[serde(rename_all = "camelCase")]
    OnPanGesture {
        window_id: u32,
        delta: Vector2,
        phase: TouchPhase,
    },

    /// Rotation gesture
    #[serde(rename_all = "camelCase")]
    OnRotationGesture {
        window_id: u32,
        delta: f32,
        phase: TouchPhase,
    },

    /// Double tap gesture
    #[serde(rename_all = "camelCase")]
    OnDoubleTapGesture { window_id: u32 },
}
