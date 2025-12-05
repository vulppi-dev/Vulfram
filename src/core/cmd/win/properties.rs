use glam::{IVec2, UVec2};
use serde::{Deserialize, Serialize};
use winit::dpi::{PhysicalPosition, PhysicalSize};

use crate::core::buffers::BufferData;
use crate::core::state::EngineState;

use super::EngineWindowState;

// MARK: - Set Title

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetTitleArgs {
    pub window_id: u32,
    pub title: String,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetTitle {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_set_title(
    engine: &mut EngineState,
    args: &CmdWindowSetTitleArgs,
) -> CmdResultWindowSetTitle {
    match engine.windows.get(&args.window_id) {
        Some(window_state) => {
            window_state.window.set_title(&args.title);
            CmdResultWindowSetTitle {
                success: true,
                message: "Title set successfully".to_string(),
            }
        }
        None => CmdResultWindowSetTitle {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

// MARK: - Set Position

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetPositionArgs {
    pub window_id: u32,
    pub position: IVec2,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetPosition {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_set_position(
    engine: &mut EngineState,
    args: &CmdWindowSetPositionArgs,
) -> CmdResultWindowSetPosition {
    match engine.windows.get(&args.window_id) {
        Some(window_state) => {
            let position = PhysicalPosition::new(args.position[0], args.position[1]);
            window_state.window.set_outer_position(position);
            CmdResultWindowSetPosition {
                success: true,
                message: "Position set successfully".to_string(),
            }
        }
        None => CmdResultWindowSetPosition {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

// MARK: - Get Position

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowGetPositionArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowGetPosition {
    success: bool,
    message: String,
    content: IVec2,
}

pub fn engine_cmd_window_get_position(
    engine: &EngineState,
    args: &CmdWindowGetPositionArgs,
) -> CmdResultWindowGetPosition {
    match engine.windows.get(&args.window_id) {
        Some(window_state) => match window_state.window.outer_position() {
            Ok(position) => CmdResultWindowGetPosition {
                success: true,
                message: "Position retrieved successfully".to_string(),
                content: IVec2::new(position.x, position.y),
            },
            Err(e) => CmdResultWindowGetPosition {
                success: false,
                message: format!("Failed to get position: {}", e),
                content: IVec2::new(0, 0),
            },
        },
        None => CmdResultWindowGetPosition {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            content: IVec2::new(0, 0),
        },
    }
}

// MARK: - Set Size

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetSizeArgs {
    pub window_id: u32,
    pub size: UVec2,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetSize {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_set_size(
    engine: &mut EngineState,
    args: &CmdWindowSetSizeArgs,
) -> CmdResultWindowSetSize {
    match engine.windows.get_mut(&args.window_id) {
        Some(window_state) => {
            let size = PhysicalSize::new(args.size[0], args.size[1]);
            let _ = window_state.window.request_inner_size(size);

            // Update surface configuration
            window_state.config.width = args.size[0];
            window_state.config.height = args.size[1];
            window_state
                .surface
                .configure(engine.device.as_ref().unwrap(), &window_state.config);

            // Mark window as dirty to trigger redraw
            window_state.is_dirty = true;

            CmdResultWindowSetSize {
                success: true,
                message: "Size set successfully".to_string(),
            }
        }
        None => CmdResultWindowSetSize {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

// MARK: - Get Size

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowGetSizeArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowGetSize {
    success: bool,
    message: String,
    content: UVec2,
}

pub fn engine_cmd_window_get_size(
    engine: &EngineState,
    args: &CmdWindowGetSizeArgs,
) -> CmdResultWindowGetSize {
    match engine.windows.get(&args.window_id) {
        Some(window_state) => {
            let size = window_state.window.inner_size();
            CmdResultWindowGetSize {
                success: true,
                message: "Size retrieved successfully".to_string(),
                content: UVec2::new(size.width, size.height),
            }
        }
        None => CmdResultWindowGetSize {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            content: UVec2::new(0, 0),
        },
    }
}

// MARK: - Get Outer Size

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowGetOuterSizeArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowGetOuterSize {
    success: bool,
    message: String,
    content: UVec2,
}

pub fn engine_cmd_window_get_outer_size(
    engine: &EngineState,
    args: &CmdWindowGetOuterSizeArgs,
) -> CmdResultWindowGetOuterSize {
    match engine.windows.get(&args.window_id) {
        Some(window_state) => {
            let size = window_state.window.outer_size();
            CmdResultWindowGetOuterSize {
                success: true,
                message: "Outer size retrieved successfully".to_string(),
                content: UVec2::new(size.width, size.height),
            }
        }
        None => CmdResultWindowGetOuterSize {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            content: UVec2::new(0, 0),
        },
    }
}

// MARK: - Get Surface Size

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowGetSurfaceSizeArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowGetSurfaceSize {
    success: bool,
    message: String,
    content: UVec2,
}

pub fn engine_cmd_window_get_surface_size(
    engine: &EngineState,
    args: &CmdWindowGetSurfaceSizeArgs,
) -> CmdResultWindowGetSurfaceSize {
    match engine.windows.get(&args.window_id) {
        Some(window_state) => CmdResultWindowGetSurfaceSize {
            success: true,
            message: "Surface size retrieved successfully".to_string(),
            content: UVec2::new(window_state.config.width, window_state.config.height),
        },
        None => CmdResultWindowGetSurfaceSize {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            content: UVec2::new(0, 0),
        },
    }
}

// MARK: - Set State

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetStateArgs {
    pub window_id: u32,
    pub state: EngineWindowState,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetState {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_set_state(
    engine: &mut EngineState,
    args: &CmdWindowSetStateArgs,
) -> CmdResultWindowSetState {
    match engine.windows.get(&args.window_id) {
        Some(window_state) => {
            match args.state {
                EngineWindowState::Minimized => {
                    window_state.window.set_minimized(true);
                }
                EngineWindowState::Maximized => {
                    window_state.window.set_maximized(true);
                }
                EngineWindowState::Windowed => {
                    window_state.window.set_minimized(false);
                    window_state.window.set_maximized(false);
                    window_state.window.set_fullscreen(None);
                }
                EngineWindowState::Fullscreen => {
                    // Exclusive fullscreen - requires video mode
                    if let Some(monitor) = window_state.window.current_monitor() {
                        if let Some(video_mode) = monitor.video_modes().next() {
                            let fullscreen = Some(winit::window::Fullscreen::Exclusive(video_mode));
                            window_state.window.set_fullscreen(fullscreen);
                        }
                    }
                }
                EngineWindowState::WindowedFullscreen => {
                    // Borderless fullscreen
                    let monitor = window_state.window.current_monitor();
                    let fullscreen = Some(winit::window::Fullscreen::Borderless(monitor));
                    window_state.window.set_fullscreen(fullscreen);
                }
            }
            CmdResultWindowSetState {
                success: true,
                message: "Window state set successfully".to_string(),
            }
        }
        None => CmdResultWindowSetState {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

// MARK: - Get State

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowGetStateArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowGetState {
    success: bool,
    message: String,
    content: EngineWindowState,
}

pub fn engine_cmd_window_get_state(
    engine: &EngineState,
    args: &CmdWindowGetStateArgs,
) -> CmdResultWindowGetState {
    match engine.windows.get(&args.window_id) {
        Some(window_state) => {
            let state = if window_state.window.is_minimized().unwrap_or(false) {
                EngineWindowState::Minimized
            } else if window_state.window.is_maximized() {
                EngineWindowState::Maximized
            } else if window_state.window.fullscreen().is_some() {
                // Check fullscreen type
                match window_state.window.fullscreen() {
                    Some(winit::window::Fullscreen::Exclusive(_)) => EngineWindowState::Fullscreen,
                    Some(winit::window::Fullscreen::Borderless(_)) => {
                        EngineWindowState::WindowedFullscreen
                    }
                    None => EngineWindowState::Windowed,
                }
            } else {
                EngineWindowState::Windowed
            };

            CmdResultWindowGetState {
                success: true,
                message: "Window state retrieved successfully".to_string(),
                content: state,
            }
        }
        None => CmdResultWindowGetState {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
            content: EngineWindowState::default(),
        },
    }
}

// MARK: - Set Icon

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowSetIconArgs {
    pub window_id: u32,
    pub buffer_id: u64,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowSetIcon {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_set_icon(
    engine: &mut EngineState,
    args: &CmdWindowSetIconArgs,
) -> CmdResultWindowSetIcon {
    // Check if window exists
    if !engine.windows.contains_key(&args.window_id) {
        return CmdResultWindowSetIcon {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        };
    }

    // Get buffer
    let buffer = match engine.buffers.get(&args.buffer_id) {
        Some(b) => b,
        None => {
            return CmdResultWindowSetIcon {
                success: false,
                message: format!("Buffer with id {} not found", args.buffer_id),
            };
        }
    };

    // Check if buffer is an image
    let image_buffer = match &buffer.data {
        BufferData::Image(img) => img,
        BufferData::Raw(_) => {
            return CmdResultWindowSetIcon {
                success: false,
                message: "Buffer is not a valid image. Supported formats: PNG, JPEG, WebP, AVIF"
                    .to_string(),
            };
        }
    };

    // Create winit icon (requires RGBA8 format)
    let icon = match winit::window::Icon::from_rgba(
        image_buffer.data.clone(),
        image_buffer.width,
        image_buffer.height,
    ) {
        Ok(icon) => icon,
        Err(e) => {
            return CmdResultWindowSetIcon {
                success: false,
                message: format!("Failed to create icon: {}", e),
            };
        }
    };

    // Apply icon to window
    let window_state = engine.windows.get(&args.window_id).unwrap();
    window_state.window.set_window_icon(Some(icon));

    CmdResultWindowSetIcon {
        success: true,
        message: "Icon set successfully".to_string(),
    }
}
