use serde::{Deserialize, Serialize};

use crate::core::state::EngineState;

// MARK: - Request Attention

/// User attention request types
#[repr(u8)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum UserAttentionType {
    /// Urgent attention (e.g., bouncing icon, flashing taskbar continuously)
    Critical = 0,
    /// Informational attention (e.g., single bounce, flash once)
    Informational = 1,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowRequestAttentionArgs {
    pub window_id: u32,
    pub attention_type: Option<UserAttentionType>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowRequestAttention {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_request_attention(
    engine: &mut EngineState,
    args: &CmdWindowRequestAttentionArgs,
) -> CmdResultWindowRequestAttention {
    match engine.windows.get(&args.window_id) {
        Some(window_state) => {
            let attention_type = args.attention_type.map(|t| match t {
                UserAttentionType::Critical => winit::window::UserAttentionType::Critical,
                UserAttentionType::Informational => winit::window::UserAttentionType::Informational,
            });
            window_state.window.request_user_attention(attention_type);
            CmdResultWindowRequestAttention {
                success: true,
                message: "User attention requested successfully".to_string(),
            }
        }
        None => CmdResultWindowRequestAttention {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}

// MARK: - Focus Window

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowFocusArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowFocus {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_focus(
    engine: &mut EngineState,
    args: &CmdWindowFocusArgs,
) -> CmdResultWindowFocus {
    match engine.windows.get(&args.window_id) {
        Some(window_state) => {
            window_state.window.focus_window();
            CmdResultWindowFocus {
                success: true,
                message: "Window focused successfully".to_string(),
            }
        }
        None => CmdResultWindowFocus {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        },
    }
}
