use serde::{Deserialize, Serialize};

use crate::core::units::{IVector2, Size};

// MARK: Window

#[repr(u32)]
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum EngineWindowState {
    Minimized = 0,
    Maximized,
    Windowed,
    Fullscreen,
    WindowedFullscreen,
}

impl Default for EngineWindowState {
    fn default() -> Self {
        EngineWindowState::Windowed
    }
}

fn window_size_default() -> Size {
    [800, 600]
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowCreateArgs {
    pub title: String,
    #[serde(default = "window_size_default")]
    pub size: Size,
    pub position: IVector2,
    pub borderless: bool,
    pub resizable: bool,
    pub always_on_top: bool,
    pub initial_state: EngineWindowState,
}
