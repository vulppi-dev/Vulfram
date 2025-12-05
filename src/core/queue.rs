use std::time::Instant;

use crate::core::cmd::engine_process_batch;

use super::cmd::EngineBatchCmds;
use super::result::VulframResult;
use super::singleton::{with_engine, with_engine_singleton};

/// Send a batch of commands to the engine
pub fn engine_send_queue(ptr: *const u8, length: usize) -> VulframResult {
    let data = unsafe { std::slice::from_raw_parts(ptr, length).to_vec() };

    let batch = match rmp_serde::from_slice::<EngineBatchCmds>(&data) {
        Err(_e) => {
            return VulframResult::CmdInvalidMessagePackError;
        }
        Ok(batch) => batch,
    };

    match with_engine_singleton(|engine| {
        engine_process_batch(&mut engine.state, engine.proxy.as_mut().unwrap(), batch)
    }) {
        Err(e) => return e,
        Ok(r) => r,
    }
}

/// Receive a batch of command responses from the engine
pub fn engine_receive_queue(out_ptr: *mut *const u8, out_length: *mut usize) -> VulframResult {
    match with_engine(|engine| {
        if engine.response_queue.is_empty() {
            unsafe {
                *out_length = 0;
                *out_ptr = std::ptr::null();
            }
            engine.profiling.serialization_ns = 0;
            return VulframResult::Success;
        }

        // MARK: Serialization
        let serialization_start = Instant::now();
        engine.serialized_responses_buffer = match rmp_serde::to_vec_named(&engine.response_queue) {
            Ok(data) => data,
            Err(_) => return VulframResult::UnknownError,
        };
        engine.profiling.serialization_ns = serialization_start.elapsed().as_nanos() as u64;

        unsafe {
            *out_ptr = engine.serialized_responses_buffer.as_ptr();
            *out_length = engine.serialized_responses_buffer.len();
        }

        engine.response_queue.clear();
        VulframResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}

/// Receive a batch of spontaneous events from the engine
pub fn engine_receive_events(out_ptr: *mut *const u8, out_length: *mut usize) -> VulframResult {
    match with_engine(|engine| {
        if engine.event_queue.is_empty() {
            unsafe {
                *out_length = 0;
                *out_ptr = std::ptr::null();
            }
            return VulframResult::Success;
        }

        // MARK: Serialization
        let serialization_start = Instant::now();
        engine.serialized_events_buffer = match rmp_serde::to_vec_named(&engine.event_queue) {
            Ok(data) => data,
            Err(_) => return VulframResult::UnknownError,
        };
        let serialization_time = serialization_start.elapsed().as_nanos() as u64;

        // Only update profiling if we're serializing responses too
        // (to avoid overwriting response serialization time)
        if engine.profiling.serialization_ns == 0 {
            engine.profiling.serialization_ns = serialization_time;
        } else {
            engine.profiling.serialization_ns += serialization_time;
        }

        unsafe {
            *out_ptr = engine.serialized_events_buffer.as_ptr();
            *out_length = engine.serialized_events_buffer.len();
        }

        engine.event_queue.clear();
        VulframResult::Success
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}
