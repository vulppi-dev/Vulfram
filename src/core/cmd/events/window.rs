use glam::IVec2;
use serde::{Deserialize, Serialize};

/// Window-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum WindowEvent {
    /// Window was created successfully
    #[serde(rename_all = "camelCase")]
    OnCreate { window_id: u32 },

    /// Window was resized
    #[serde(rename_all = "camelCase")]
    OnResize {
        window_id: u32,
        width: u32,
        height: u32,
    },

    /// Window was moved
    #[serde(rename_all = "camelCase")]
    OnMove { window_id: u32, position: IVec2 },

    /// Window close was requested by user
    #[serde(rename_all = "camelCase")]
    OnCloseRequest { window_id: u32 },

    /// Window was destroyed
    #[serde(rename_all = "camelCase")]
    OnDestroy { window_id: u32 },

    /// Window gained or lost focus
    #[serde(rename_all = "camelCase")]
    OnFocus { window_id: u32, focused: bool },

    /// Window scale factor changed
    #[serde(rename_all = "camelCase")]
    OnScaleFactorChange {
        window_id: u32,
        scale_factor: f64,
        new_width: u32,
        new_height: u32,
    },

    /// Window was occluded (completely hidden from view)
    #[serde(rename_all = "camelCase")]
    OnOcclude { window_id: u32, occluded: bool },

    /// Window redraw was requested
    #[serde(rename_all = "camelCase")]
    OnRedrawRequest { window_id: u32 },

    /// File was dropped into window
    #[serde(rename_all = "camelCase")]
    OnFileDrop { window_id: u32, path: String },

    /// File is being hovered over window
    #[serde(rename_all = "camelCase")]
    OnFileHover { window_id: u32, path: String },

    /// Hovered file left the window
    #[serde(rename_all = "camelCase")]
    OnFileHoverCancel { window_id: u32 },

    /// System theme changed
    #[serde(rename_all = "camelCase")]
    OnThemeChange { window_id: u32, dark_mode: bool },
}
