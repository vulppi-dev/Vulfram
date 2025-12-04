use super::result::VulframResult;
use super::singleton::with_engine;

/// Profiling data structure for export
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfilingData {
    pub gamepad_processing_us: f64,
    pub event_loop_pump_us: f64,
    pub request_redraw_us: f64,
    pub serialization_us: f64,
    pub total_events_dispatched: usize,
    pub total_events_cached: usize,
}

/// Get detailed profiling data from the last tick
pub fn engine_get_profiling(out_ptr: *mut *const u8, out_length: *mut usize) -> VulframResult {
    match with_engine(|engine| {
        let data = ProfilingData {
            gamepad_processing_us: engine.profiling.gamepad_processing_ns as f64 / 1000.0,
            event_loop_pump_us: engine.profiling.event_loop_pump_ns as f64 / 1000.0,
            request_redraw_us: engine.profiling.request_redraw_ns as f64 / 1000.0,
            serialization_us: engine.profiling.serialization_ns as f64 / 1000.0,
            total_events_dispatched: engine.profiling.total_events_dispatched,
            total_events_cached: engine.profiling.total_events_cached,
        };

        // Serialize profiling data
        engine.serialized_events_buffer = match rmp_serde::to_vec_named(&data) {
            Ok(data) => data,
            Err(_) => return VulframResult::UnknownError,
        };

        unsafe {
            *out_ptr = engine.serialized_events_buffer.as_ptr();
            *out_length = engine.serialized_events_buffer.len();
        }

        VulframResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}
