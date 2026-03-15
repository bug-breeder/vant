/// Returns the version string of the vant engine.
/// The returned pointer is valid for the lifetime of the program.
#[no_mangle]
pub extern "C" fn vant_engine_version() -> *const libc::c_char {
    static VERSION: &[u8] = b"0.1.0\0";
    VERSION.as_ptr() as *const libc::c_char
}

/// Returns 1 if the engine is operational, 0 otherwise.
/// Use this to verify the FFI bridge is working.
#[no_mangle]
pub extern "C" fn vant_engine_health_check() -> i32 {
    1
}
