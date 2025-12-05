mod core;

// ============================================================================
// N-API Exports - for Node.js native modules
// ============================================================================
#[cfg(feature = "napi")]
#[allow(dead_code)]
mod napi_exports {
    use super::core;
    use napi::bindgen_prelude::*;
    use napi_derive::napi;

    #[napi(object)]
    pub struct BufferResult {
        pub buffer: Buffer,
        pub result: u32,
    }

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
    pub fn engine_receive_queue() -> Result<BufferResult> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::engine_receive_queue(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok(BufferResult {
                buffer: Buffer::from(vec![]),
                result,
            });
        }

        // Copy data from internal buffer
        let data = unsafe { std::slice::from_raw_parts(ptr, length) };

        Ok(BufferResult {
            buffer: Buffer::from(data.to_vec()),
            result,
        })
    }

    #[napi]
    pub fn engine_receive_events() -> Result<BufferResult> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::engine_receive_events(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok(BufferResult {
                buffer: Buffer::from(vec![]),
                result,
            });
        }

        // Copy data from internal buffer
        let data = unsafe { std::slice::from_raw_parts(ptr, length) };

        Ok(BufferResult {
            buffer: Buffer::from(data.to_vec()),
            result,
        })
    }

    #[napi]
    pub fn engine_upload_buffer(id: i64, data: Buffer) -> u32 {
        let ptr = data.as_ptr();
        let length = data.len();
        core::engine_upload_buffer(id as u64, ptr, length) as u32
    }

    #[napi]
    pub fn engine_download_buffer(id: i64) -> Result<BufferResult> {
        let mut length: usize = 0;
        let length_ptr = &mut length as *mut usize;

        // First call to get size
        let result =
            core::engine_download_buffer(id as u64, std::ptr::null_mut(), length_ptr) as u32;
        if result != 0 || length == 0 {
            return Ok(BufferResult {
                buffer: Buffer::from(vec![]),
                result,
            });
        }

        // Second call to get data
        let mut buffer = vec![0u8; length];
        let buffer_ptr = buffer.as_mut_ptr();
        let result = core::engine_download_buffer(id as u64, buffer_ptr, length_ptr) as u32;

        if result != 0 {
            return Ok(BufferResult {
                buffer: Buffer::from(vec![]),
                result,
            });
        }

        Ok(BufferResult {
            buffer: Buffer::from(buffer),
            result,
        })
    }

    #[napi]
    pub fn engine_clear_buffer(id: i64) -> u32 {
        core::engine_clear_buffer(id as u64) as u32
    }

    #[napi]
    pub fn engine_tick(time: i64, delta_time: u32) -> u32 {
        core::engine_tick(time as u64, delta_time) as u32
    }

    #[napi]
    pub fn engine_get_profiling() -> Result<BufferResult> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::engine_get_profiling(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok(BufferResult {
                buffer: Buffer::from(vec![]),
                result,
            });
        }

        // Copy data from internal buffer
        let data = unsafe { std::slice::from_raw_parts(ptr, length) };

        Ok(BufferResult {
            buffer: Buffer::from(data.to_vec()),
            result,
        })
    }
}

// ============================================================================
// Lua Exports - for Lua bindings via mlua
// ============================================================================
#[cfg(feature = "lua")]
#[allow(dead_code)]
mod lua_exports {
    use super::core;
    use mlua::prelude::*;

    fn engine_init(_: &Lua, _: ()) -> LuaResult<u32> {
        Ok(core::engine_init() as u32)
    }

    fn engine_dispose(_: &Lua, _: ()) -> LuaResult<u32> {
        Ok(core::engine_dispose() as u32)
    }

    fn engine_send_queue(_: &Lua, data: LuaString) -> LuaResult<u32> {
        let bytes = data.as_bytes();
        Ok(core::engine_send_queue(bytes.as_ptr(), bytes.len()) as u32)
    }

    fn engine_receive_queue(lua: &Lua, _: ()) -> LuaResult<(LuaString, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::engine_receive_queue(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((lua.create_string(&[])?, result));
        }

        let data = unsafe { std::slice::from_raw_parts(ptr, length) };
        Ok((lua.create_string(data)?, result))
    }

    fn engine_receive_events(lua: &Lua, _: ()) -> LuaResult<(LuaString, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::engine_receive_events(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((lua.create_string(&[])?, result));
        }

        let data = unsafe { std::slice::from_raw_parts(ptr, length) };
        Ok((lua.create_string(data)?, result))
    }

    fn engine_upload_buffer(_: &Lua, (id, data): (i64, LuaString)) -> LuaResult<u32> {
        let bytes = data.as_bytes();
        Ok(core::engine_upload_buffer(id as u64, bytes.as_ptr(), bytes.len()) as u32)
    }

    fn engine_download_buffer(lua: &Lua, id: i64) -> LuaResult<(LuaString, u32)> {
        let mut length: usize = 0;
        let length_ptr = &mut length as *mut usize;

        // First call to get size
        let result =
            core::engine_download_buffer(id as u64, std::ptr::null_mut(), length_ptr) as u32;
        if result != 0 || length == 0 {
            return Ok((lua.create_string(&[])?, result));
        }

        // Second call to get data
        let mut buffer = vec![0u8; length];
        let buffer_ptr = buffer.as_mut_ptr();
        let result = core::engine_download_buffer(id as u64, buffer_ptr, length_ptr) as u32;

        if result != 0 {
            return Ok((lua.create_string(&[])?, result));
        }

        Ok((lua.create_string(&buffer)?, result))
    }

    fn engine_clear_buffer(_: &Lua, id: i64) -> LuaResult<u32> {
        Ok(core::engine_clear_buffer(id as u64) as u32)
    }

    fn engine_tick(_: &Lua, (time, delta_time): (i64, u32)) -> LuaResult<u32> {
        Ok(core::engine_tick(time as u64, delta_time) as u32)
    }

    fn engine_get_profiling(lua: &Lua, _: ()) -> LuaResult<(LuaString, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::engine_get_profiling(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((lua.create_string(&[])?, result));
        }

        let data = unsafe { std::slice::from_raw_parts(ptr, length) };
        Ok((lua.create_string(data)?, result))
    }

    #[mlua::lua_module]
    pub fn vulfram(lua: &Lua) -> LuaResult<LuaTable> {
        let exports = lua.create_table()?;
        exports.set("init", lua.create_function(engine_init)?)?;
        exports.set("dispose", lua.create_function(engine_dispose)?)?;
        exports.set("send_queue", lua.create_function(engine_send_queue)?)?;
        exports.set("receive_queue", lua.create_function(engine_receive_queue)?)?;
        exports.set(
            "receive_events",
            lua.create_function(engine_receive_events)?,
        )?;
        exports.set("upload_buffer", lua.create_function(engine_upload_buffer)?)?;
        exports.set(
            "download_buffer",
            lua.create_function(engine_download_buffer)?,
        )?;
        exports.set("clear_buffer", lua.create_function(engine_clear_buffer)?)?;
        exports.set("tick", lua.create_function(engine_tick)?)?;
        exports.set("get_profiling", lua.create_function(engine_get_profiling)?)?;
        Ok(exports)
    }
}

// ============================================================================
// Python Exports - for Python bindings via PyO3
// ============================================================================
#[cfg(feature = "python")]
#[allow(dead_code)]
mod python_exports {
    use super::core;
    use pyo3::prelude::*;
    use pyo3::types::PyBytes;

    #[pyfunction]
    fn engine_init() -> u32 {
        core::engine_init() as u32
    }

    #[pyfunction]
    fn engine_dispose() -> u32 {
        core::engine_dispose() as u32
    }

    #[pyfunction]
    fn engine_send_queue(data: &[u8]) -> u32 {
        core::engine_send_queue(data.as_ptr(), data.len()) as u32
    }

    #[pyfunction]
    fn engine_receive_queue(py: Python) -> PyResult<(Py<PyBytes>, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::engine_receive_queue(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((PyBytes::new(py, &[]).into(), result));
        }

        let data = unsafe { std::slice::from_raw_parts(ptr, length) };
        Ok((PyBytes::new(py, data).into(), result))
    }

    #[pyfunction]
    fn engine_receive_events(py: Python) -> PyResult<(Py<PyBytes>, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::engine_receive_events(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((PyBytes::new(py, &[]).into(), result));
        }

        let data = unsafe { std::slice::from_raw_parts(ptr, length) };
        Ok((PyBytes::new(py, data).into(), result))
    }

    #[pyfunction]
    fn engine_upload_buffer(id: i64, data: &[u8]) -> u32 {
        core::engine_upload_buffer(id as u64, data.as_ptr(), data.len()) as u32
    }

    #[pyfunction]
    fn engine_download_buffer(py: Python, id: i64) -> PyResult<(Py<PyBytes>, u32)> {
        let mut length: usize = 0;
        let length_ptr = &mut length as *mut usize;

        // First call to get size
        let result =
            core::engine_download_buffer(id as u64, std::ptr::null_mut(), length_ptr) as u32;
        if result != 0 || length == 0 {
            return Ok((PyBytes::new(py, &[]).into(), result));
        }

        // Second call to get data
        let mut buffer = vec![0u8; length];
        let buffer_ptr = buffer.as_mut_ptr();
        let result = core::engine_download_buffer(id as u64, buffer_ptr, length_ptr) as u32;

        if result != 0 {
            return Ok((PyBytes::new(py, &[]).into(), result));
        }

        Ok((PyBytes::new(py, &buffer).into(), result))
    }

    #[pyfunction]
    fn engine_clear_buffer(id: i64) -> u32 {
        core::engine_clear_buffer(id as u64) as u32
    }

    #[pyfunction]
    fn engine_tick(time: i64, delta_time: u32) -> u32 {
        core::engine_tick(time as u64, delta_time) as u32
    }

    #[pyfunction]
    fn engine_get_profiling(py: Python) -> PyResult<(Py<PyBytes>, u32)> {
        let mut length: usize = 0;
        let mut ptr: *const u8 = std::ptr::null();
        let length_ptr = &mut length as *mut usize;
        let ptr_ptr = &mut ptr as *mut *const u8;

        let result = core::engine_get_profiling(ptr_ptr, length_ptr) as u32;

        if result != 0 || length == 0 {
            return Ok((PyBytes::new(py, &[]).into(), result));
        }

        let data = unsafe { std::slice::from_raw_parts(ptr, length) };
        Ok((PyBytes::new(py, data).into(), result))
    }

    #[pymodule]
    fn vulfram(module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add_function(wrap_pyfunction!(engine_init, module)?)?;
        module.add_function(wrap_pyfunction!(engine_dispose, module)?)?;
        module.add_function(wrap_pyfunction!(engine_send_queue, module)?)?;
        module.add_function(wrap_pyfunction!(engine_receive_queue, module)?)?;
        module.add_function(wrap_pyfunction!(engine_receive_events, module)?)?;
        module.add_function(wrap_pyfunction!(engine_upload_buffer, module)?)?;
        module.add_function(wrap_pyfunction!(engine_download_buffer, module)?)?;
        module.add_function(wrap_pyfunction!(engine_clear_buffer, module)?)?;
        module.add_function(wrap_pyfunction!(engine_tick, module)?)?;
        module.add_function(wrap_pyfunction!(engine_get_profiling, module)?)?;
        Ok(())
    }
}

// ============================================================================
// FFI (C ABI) Exports - for Bun/Node FFI via dlopen
// ============================================================================
#[cfg(feature = "ffi")]
#[allow(dead_code)]
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
    pub extern "C" fn engine_receive_queue(out_ptr: *mut *const u8, out_length: *mut usize) -> u32 {
        core::engine_receive_queue(out_ptr, out_length) as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn engine_receive_events(
        out_ptr: *mut *const u8,
        out_length: *mut usize,
    ) -> u32 {
        core::engine_receive_events(out_ptr, out_length) as u32
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

    #[unsafe(no_mangle)]
    pub extern "C" fn engine_get_profiling(out_ptr: *mut *const u8, out_length: *mut usize) -> u32 {
        core::engine_get_profiling(out_ptr, out_length) as u32
    }
}
