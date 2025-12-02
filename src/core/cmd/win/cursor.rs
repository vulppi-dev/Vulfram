use serde::{Deserialize, Serialize};

use crate::core::state::EngineState;

// MARK: - Set Cursor Visible

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetCursorVisibleArgs {
    pub window_id: u32,
    pub visible: bool,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetCursorVisible {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_set_cursor_visible(
    engine: &mut EngineState,
    args: &CmdWindowSetCursorVisibleArgs,
) -> CmdResultWindowSetCursorVisible {
    match engine.windows.get(&args.window_id) {
        Some(window_state) => {
            window_state.window.set_cursor_visible(args.visible);
            CmdResultWindowSetCursorVisible {
                success: true,
                message: "Cursor visibility set successfully".to_string(),
            }
        }
        None => CmdResultWindowSetCursorVisible {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

// MARK: - Set Cursor Grab

/// Cursor grab modes
#[repr(u8)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum CursorGrabMode {
    /// No grabbing
    None = 0,
    /// Cursor confined to window
    Confined = 1,
    /// Cursor locked to window
    Locked = 2,
}

impl Default for CursorGrabMode {
    fn default() -> Self {
        CursorGrabMode::None
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetCursorGrabArgs {
    pub window_id: u32,
    pub mode: CursorGrabMode,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetCursorGrab {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_set_cursor_grab(
    engine: &mut EngineState,
    args: &CmdWindowSetCursorGrabArgs,
) -> CmdResultWindowSetCursorGrab {
    match engine.windows.get(&args.window_id) {
        Some(window_state) => {
            let grab_mode = match args.mode {
                CursorGrabMode::None => {
                    // Release grab
                    match window_state
                        .window
                        .set_cursor_grab(winit::window::CursorGrabMode::None)
                    {
                        Ok(_) => {
                            return CmdResultWindowSetCursorGrab {
                                success: true,
                                message: "Cursor grab released successfully".to_string(),
                            };
                        }
                        Err(e) => {
                            return CmdResultWindowSetCursorGrab {
                                success: false,
                                message: format!("Failed to release cursor grab: {}", e),
                            };
                        }
                    }
                }
                CursorGrabMode::Confined => winit::window::CursorGrabMode::Confined,
                CursorGrabMode::Locked => winit::window::CursorGrabMode::Locked,
            };

            match window_state.window.set_cursor_grab(grab_mode) {
                Ok(_) => CmdResultWindowSetCursorGrab {
                    success: true,
                    message: "Cursor grab mode set successfully".to_string(),
                },
                Err(e) => CmdResultWindowSetCursorGrab {
                    success: false,
                    message: format!("Failed to set cursor grab mode: {}", e),
                },
            }
        }
        None => CmdResultWindowSetCursorGrab {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

// MARK: - Set Cursor Icon

/// Cursor icon types
#[repr(u16)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum CursorIcon {
    /// Default cursor
    Default = 0,
    /// Context menu cursor
    ContextMenu = 1,
    /// Help cursor
    Help = 2,
    /// Pointer/hand cursor
    Pointer = 3,
    /// Progress cursor
    Progress = 4,
    /// Wait cursor
    Wait = 5,
    /// Cell cursor
    Cell = 6,
    /// Crosshair cursor
    Crosshair = 7,
    /// Text cursor
    Text = 8,
    /// Vertical text cursor
    VerticalText = 9,
    /// Alias cursor
    Alias = 10,
    /// Copy cursor
    Copy = 11,
    /// Move cursor
    Move = 12,
    /// No drop cursor
    NoDrop = 13,
    /// Not allowed cursor
    NotAllowed = 14,
    /// Grab cursor
    Grab = 15,
    /// Grabbing cursor
    Grabbing = 16,
    /// E-resize cursor
    EResize = 17,
    /// N-resize cursor
    NResize = 18,
    /// NE-resize cursor
    NeResize = 19,
    /// NW-resize cursor
    NwResize = 20,
    /// S-resize cursor
    SResize = 21,
    /// SE-resize cursor
    SeResize = 22,
    /// SW-resize cursor
    SwResize = 23,
    /// W-resize cursor
    WResize = 24,
    /// EW-resize cursor
    EwResize = 25,
    /// NS-resize cursor
    NsResize = 26,
    /// NESW-resize cursor
    NeswResize = 27,
    /// NWSE-resize cursor
    NwseResize = 28,
    /// Column resize cursor
    ColResize = 29,
    /// Row resize cursor
    RowResize = 30,
    /// All scroll cursor
    AllScroll = 31,
    /// Zoom in cursor
    ZoomIn = 32,
    /// Zoom out cursor
    ZoomOut = 33,
}

impl Default for CursorIcon {
    fn default() -> Self {
        CursorIcon::Default
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetCursorIconArgs {
    pub window_id: u32,
    pub icon: CursorIcon,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetCursorIcon {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_set_cursor_icon(
    engine: &mut EngineState,
    args: &CmdWindowSetCursorIconArgs,
) -> CmdResultWindowSetCursorIcon {
    match engine.windows.get(&args.window_id) {
        Some(window_state) => {
            let winit_icon = match args.icon {
                CursorIcon::Default => winit::window::CursorIcon::Default,
                CursorIcon::ContextMenu => winit::window::CursorIcon::ContextMenu,
                CursorIcon::Help => winit::window::CursorIcon::Help,
                CursorIcon::Pointer => winit::window::CursorIcon::Pointer,
                CursorIcon::Progress => winit::window::CursorIcon::Progress,
                CursorIcon::Wait => winit::window::CursorIcon::Wait,
                CursorIcon::Cell => winit::window::CursorIcon::Cell,
                CursorIcon::Crosshair => winit::window::CursorIcon::Crosshair,
                CursorIcon::Text => winit::window::CursorIcon::Text,
                CursorIcon::VerticalText => winit::window::CursorIcon::VerticalText,
                CursorIcon::Alias => winit::window::CursorIcon::Alias,
                CursorIcon::Copy => winit::window::CursorIcon::Copy,
                CursorIcon::Move => winit::window::CursorIcon::Move,
                CursorIcon::NoDrop => winit::window::CursorIcon::NoDrop,
                CursorIcon::NotAllowed => winit::window::CursorIcon::NotAllowed,
                CursorIcon::Grab => winit::window::CursorIcon::Grab,
                CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
                CursorIcon::EResize => winit::window::CursorIcon::EResize,
                CursorIcon::NResize => winit::window::CursorIcon::NResize,
                CursorIcon::NeResize => winit::window::CursorIcon::NeResize,
                CursorIcon::NwResize => winit::window::CursorIcon::NwResize,
                CursorIcon::SResize => winit::window::CursorIcon::SResize,
                CursorIcon::SeResize => winit::window::CursorIcon::SeResize,
                CursorIcon::SwResize => winit::window::CursorIcon::SwResize,
                CursorIcon::WResize => winit::window::CursorIcon::WResize,
                CursorIcon::EwResize => winit::window::CursorIcon::EwResize,
                CursorIcon::NsResize => winit::window::CursorIcon::NsResize,
                CursorIcon::NeswResize => winit::window::CursorIcon::NeswResize,
                CursorIcon::NwseResize => winit::window::CursorIcon::NwseResize,
                CursorIcon::ColResize => winit::window::CursorIcon::ColResize,
                CursorIcon::RowResize => winit::window::CursorIcon::RowResize,
                CursorIcon::AllScroll => winit::window::CursorIcon::AllScroll,
                CursorIcon::ZoomIn => winit::window::CursorIcon::ZoomIn,
                CursorIcon::ZoomOut => winit::window::CursorIcon::ZoomOut,
            };

            window_state.window.set_cursor(winit_icon);
            CmdResultWindowSetCursorIcon {
                success: true,
                message: "Cursor icon set successfully".to_string(),
            }
        }
        None => CmdResultWindowSetCursorIcon {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}
