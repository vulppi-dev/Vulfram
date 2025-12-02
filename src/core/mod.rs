// Submodules
mod buffers;
mod handler;
mod lifecycle;
mod queue;
mod render;
mod result;
mod singleton;
mod state;
mod tick;

// Public modules
pub mod cmd;
pub mod image;
pub mod units;

// Re-exports for public API
pub use buffers::{engine_clear_buffer, engine_download_buffer, engine_upload_buffer};
pub use lifecycle::{engine_dispose, engine_init};
pub use queue::{engine_receive_queue, engine_send_queue};
pub use tick::engine_tick;

// Internal re-exports (used by lib.rs and internal modules)
#[allow(unused_imports)]
pub(crate) use result::EngineResult;
#[allow(unused_imports)]
pub(crate) use singleton::{with_engine, with_engine_singleton};
#[allow(unused_imports)]
pub(crate) use state::{EngineState, WindowState};
