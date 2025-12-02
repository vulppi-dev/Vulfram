mod core;

// ============================================================================
// FFI (C ABI) Exports - for Bun/Node FFI via dlopen
// ============================================================================
#[cfg(feature = "ffi")]
mod ffi_exports {
    use super::core;

    #[unsafe(no_mangle)]
    pub extern "C" fn engine_init() -> u32 {
        core::engine_init() as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn engine_dispose() -> u32 {
        core::engine_dispose() as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn engine_send_queue(ptr: *const u8, length: usize) -> u32 {
        core::engine_send_queue(ptr, length) as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn engine_receive_queue(out_ptr: *mut u8, out_length: *mut usize) -> u32 {
        core::engine_receive_queue(out_ptr, out_length) as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn engine_upload_buffer(
        bfr_id: u64,
        bfr_ptr: *const u8,
        bfr_length: usize,
    ) -> u32 {
        core::engine_upload_buffer(bfr_id, bfr_ptr, bfr_length) as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn engine_download_buffer(
        bfr_id: u64,
        bfr_ptr: *mut u8,
        bfr_length: *mut usize,
    ) -> u32 {
        core::engine_download_buffer(bfr_id, bfr_ptr, bfr_length) as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn engine_clear_buffer(bfr_id: u64) -> u32 {
        core::engine_clear_buffer(bfr_id) as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn engine_tick(time: u64, delta_time: u32) -> u32 {
        core::engine_tick(time, delta_time) as u32
    }
}

// ============================================================================
// N-API Exports - for Node.js native modules
// ============================================================================
#[cfg(feature = "napi")]
mod napi_exports {
    use super::core;
    use napi::bindgen_prelude::*;
    use napi_derive::napi;

    #[napi]
    pub fn engine_init() -> u32 {
        core::engine_init() as u32
    }

    #[napi]
    pub fn engine_dispose() -> u32 {
        core::engine_dispose() as u32
    }

    #[napi]
    pub fn engine_send_queue(data: Buffer) -> u32 {
        let ptr = data.as_ptr();
        let length = data.len();
        core::engine_send_queue(ptr, length) as u32
    }

    #[napi]
    pub fn engine_receive_queue() -> Result<Buffer> {
        let mut length: usize = 0;
        let length_ptr = &mut length as *mut usize;

        // First call to get size
        let result = core::engine_receive_queue(std::ptr::null_mut(), length_ptr) as u32;
        if result != 0 || length == 0 {
            return Ok(Buffer::from(vec![]));
        }

        // Second call to get data
        let mut buffer = vec![0u8; length];
        let buffer_ptr = buffer.as_mut_ptr();
        let result = core::engine_receive_queue(buffer_ptr, length_ptr) as u32;

        if result != 0 {
            return Ok(Buffer::from(vec![]));
        }

        Ok(Buffer::from(buffer))
    }

    #[napi]
    pub fn engine_upload_buffer(id: i64, data: Buffer) -> u32 {
        let ptr = data.as_ptr();
        let length = data.len();
        core::engine_upload_buffer(id as u64, ptr, length) as u32
    }

    #[napi]
    pub fn engine_download_buffer(id: i64) -> Result<Buffer> {
        let mut length: usize = 0;
        let length_ptr = &mut length as *mut usize;

        // First call to get size
        let result =
            core::engine_download_buffer(id as u64, std::ptr::null_mut(), length_ptr) as u32;
        if result != 0 || length == 0 {
            return Ok(Buffer::from(vec![]));
        }

        // Second call to get data
        let mut buffer = vec![0u8; length];
        let buffer_ptr = buffer.as_mut_ptr();
        let result = core::engine_download_buffer(id as u64, buffer_ptr, length_ptr) as u32;

        if result != 0 {
            return Ok(Buffer::from(vec![]));
        }

        Ok(Buffer::from(buffer))
    }

    #[napi]
    pub fn engine_clear_buffer(id: i64) -> u32 {
        core::engine_clear_buffer(id as u64) as u32
    }

    #[napi]
    pub fn engine_tick(time: i64, delta_time: u32) -> u32 {
        core::engine_tick(time as u64, delta_time) as u32
    }
}
