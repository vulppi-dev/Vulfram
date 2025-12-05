use serde::{Deserialize, Serialize};

mod attention;
mod create;
mod cursor;
mod decorations;
mod properties;

pub use attention::*;
pub use create::*;
pub use cursor::*;
pub use decorations::*;
pub use properties::*;

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

fn window_size_default() -> glam::UVec2 {
    glam::UVec2::new(800, 600)
}
