use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::common::{ElementState, TouchPhase};

/// Mouse scroll delta type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "kebab-case")]
pub enum ScrollDelta {
    /// Line-based scrolling (traditional mouse wheel)
    Line(Vec2),
    /// Pixel-based scrolling (touchpad)
    Pixel(Vec2),
}

/// Pointer (Mouse/Touch) events - unified for both input types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum PointerEvent {
    /// Pointer moved
    #[serde(rename_all = "camelCase")]
    OnMove {
        window_id: u32,
        pointer_type: u32,
        pointer_id: u64,
        position: Vec2,
    },

    /// Pointer entered window area
    #[serde(rename_all = "camelCase")]
    OnEnter {
        window_id: u32,
        pointer_type: u32,
        pointer_id: u64,
    },

    /// Pointer left window area
    #[serde(rename_all = "camelCase")]
    OnLeave {
        window_id: u32,
        pointer_type: u32,
        pointer_id: u64,
    },

    /// Pointer button pressed/released (mouse) or touch started/ended
    #[serde(rename_all = "camelCase")]
    OnButton {
        window_id: u32,
        pointer_type: u32,
        pointer_id: u64,
        button: u32,
        state: ElementState,
        position: Vec2,
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
        position: Vec2,
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
        delta: Vec2,
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
