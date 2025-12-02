// Window management module - organized into submodules

mod attention;
mod create;
mod cursor;
mod decorations;
mod properties;

// Re-export all public types and functions
pub use attention::*;
pub use create::*;
pub use cursor::*;
pub use decorations::*;
pub use properties::*;

use serde::{Deserialize, Serialize};

// Shared types
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

fn window_size_default() -> crate::core::units::Size {
    [800, 600]
}
