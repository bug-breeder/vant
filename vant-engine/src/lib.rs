pub mod ffi;

#[cfg(test)]
mod tests {
    use crate::ffi;
    use std::ffi::CStr;

    #[test]
    fn version_returns_valid_string() {
        let ptr = ffi::vant_engine_version();
        let version = unsafe { CStr::from_ptr(ptr) }.to_str().unwrap();
        assert_eq!(version, "0.1.0");
    }

    #[test]
    fn health_check_returns_ok() {
        assert_eq!(ffi::vant_engine_health_check(), 1);
    }
}
