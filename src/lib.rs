mod core;

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
pub extern "C" fn engine_upload_buffer(bfr_id: u64, bfr_ptr: *const u8, bfr_length: usize) -> u32 {
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
