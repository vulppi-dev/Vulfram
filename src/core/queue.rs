use super::cmd::EngineBatchCmds;
use super::result::EngineResult;
use super::singleton::{EngineCustomEvents, with_engine, with_engine_singleton};

/// Send a batch of commands to the engine
pub fn engine_send_queue(ptr: *const u8, length: usize) -> EngineResult {
    let data = unsafe { std::slice::from_raw_parts(ptr, length).to_vec() };

    let batch = match serde_cbor::from_slice::<EngineBatchCmds>(&data) {
        Err(_e) => {
            return EngineResult::CmdInvalidCborError;
        }
        Ok(batch) => batch,
    };

    match with_engine_singleton(|engine| {
        if let Some(proxy) = &engine.proxy {
            let _ = proxy.send_event(EngineCustomEvents::ProcessCommands(batch));
        }
    }) {
        Err(e) => return e,
        Ok(_) => EngineResult::Success,
    }
}

/// Receive a batch of events from the engine
pub fn engine_receive_queue(out_ptr: *mut u8, out_length: *mut usize) -> EngineResult {
    match with_engine(|engine| {
        let serialized = match serde_cbor::to_vec(&engine.event_queue) {
            Ok(data) => data,
            Err(_) => return EngineResult::UnknownError,
        };

        let required_length = serialized.len();

        unsafe {
            if out_ptr.is_null() {
                *out_length = required_length;
                return EngineResult::Success;
            }

            let available_length = *out_length;

            if required_length <= available_length {
                std::ptr::copy_nonoverlapping(serialized.as_ptr(), out_ptr, required_length);
                *out_length = required_length;
                engine.event_queue.clear();
                return EngineResult::Success;
            } else {
                *out_length = required_length;
                return EngineResult::BufferOverflow;
            }
        }
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}
