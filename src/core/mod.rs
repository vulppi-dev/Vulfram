// Submodules
mod buffers;
mod cache;
mod handler;
mod lifecycle;
mod profiling;
mod queue;
mod render;
mod result;
mod singleton;
mod state;
mod tick;

// Public modules
pub mod cmd;
pub mod image;

// Re-exports for public API
pub use buffers::{engine_clear_buffer, engine_download_buffer, engine_upload_buffer};
pub use lifecycle::{engine_dispose, engine_init};
pub use profiling::engine_get_profiling;
pub use queue::{engine_receive_events, engine_receive_queue, engine_send_queue};
pub use tick::engine_tick;
