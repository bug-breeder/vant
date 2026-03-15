pub mod engine;
pub mod event;
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

    #[test]
    fn ffi_create_destroy() {
        let engine = ffi::vant_engine_create();
        assert!(!engine.is_null());
        assert_eq!(ffi::vant_engine_is_composing(engine), 0);
        ffi::vant_engine_destroy(engine);
    }

    #[test]
    fn ffi_process_key_composing() {
        let engine = ffi::vant_engine_create();
        let result = ffi::vant_engine_process_key(engine, 'v' as u32, false, false);
        assert_eq!(result.event_type as i32, ffi::VantEventType::Composing as i32);
        assert_eq!(result.text_len, 1);
        assert_eq!(ffi::vant_engine_is_composing(engine), 1);

        let text = unsafe { CStr::from_ptr(result.text) }.to_str().unwrap();
        assert_eq!(text, "v");

        ffi::vant_engine_destroy(engine);
    }

    #[test]
    fn ffi_process_key_commit() {
        let engine = ffi::vant_engine_create();
        // Type "as" → "á"
        ffi::vant_engine_process_key(engine, 'a' as u32, false, false);
        ffi::vant_engine_process_key(engine, 's' as u32, false, false);

        // Space commits
        let result = ffi::vant_engine_process_key(engine, ' ' as u32, false, false);
        assert_eq!(result.event_type as i32, ffi::VantEventType::Committed as i32);

        let text = unsafe { CStr::from_ptr(result.text) }.to_str().unwrap();
        assert_eq!(text, "á");
        assert_eq!(result.committed_char, ' ' as u32);
        assert_eq!(ffi::vant_engine_is_composing(engine), 0);

        ffi::vant_engine_destroy(engine);
    }

    #[test]
    fn ffi_force_commit() {
        let engine = ffi::vant_engine_create();
        ffi::vant_engine_process_key(engine, 'a' as u32, false, false);

        let result = ffi::vant_engine_force_commit(engine);
        assert_eq!(result.event_type as i32, ffi::VantEventType::Committed as i32);

        let text = unsafe { CStr::from_ptr(result.text) }.to_str().unwrap();
        assert_eq!(text, "a");
        assert_eq!(result.committed_char, 0);
        assert_eq!(ffi::vant_engine_is_composing(engine), 0);

        ffi::vant_engine_destroy(engine);
    }

    #[test]
    fn ffi_reset() {
        let engine = ffi::vant_engine_create();
        ffi::vant_engine_process_key(engine, 'a' as u32, false, false);

        let result = ffi::vant_engine_reset(engine);
        assert_eq!(result.event_type as i32, ffi::VantEventType::Reset as i32);
        assert_eq!(ffi::vant_engine_is_composing(engine), 0);

        ffi::vant_engine_destroy(engine);
    }

    #[test]
    fn ffi_vietnamese_syllable() {
        let engine = ffi::vant_engine_create();
        // Type "vieetj" → "việt"
        for ch in "vieetj".chars() {
            ffi::vant_engine_process_key(engine, ch as u32, false, false);
        }

        let result = ffi::vant_engine_force_commit(engine);
        let text = unsafe { CStr::from_ptr(result.text) }.to_str().unwrap();
        assert_eq!(text, "việt");

        ffi::vant_engine_destroy(engine);
    }
}
