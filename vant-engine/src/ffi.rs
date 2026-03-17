use std::ffi::CString;

use crate::engine::TelexEngine;
use crate::event::SyllableEvent;

// ---------------------------------------------------------------------------
// Existing FFI functions
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// C-compatible types
// ---------------------------------------------------------------------------

#[repr(C)]
pub enum VantEventType {
    Composing = 0,
    Committed = 1,
    Reset = 2,
    Passthrough = 3,
}

#[repr(C)]
pub struct VantKeyResult {
    pub event_type: VantEventType,
    /// Composed or committed UTF-8 text. Valid until the next FFI call on
    /// the same engine.
    pub text: *const libc::c_char,
    /// Byte length of `text` (UTF-8). Not the number of characters.
    pub text_len: u32,
    /// Raw keystroke buffer UTF-8. Valid until the next FFI call on the same
    /// engine.
    pub raw: *const libc::c_char,
    /// Byte length of `raw` (ASCII, so bytes == characters).
    pub raw_len: u32,
    /// Trigger character codepoint, or 0 if none.
    pub committed_char: u32,
}

// ---------------------------------------------------------------------------
// Opaque engine wrapper
// ---------------------------------------------------------------------------

/// Opaque wrapper that owns the TelexEngine and CString buffers whose
/// pointers are returned to C. The pointers remain valid until the next
/// FFI call on the same `VantEngine` instance.
pub struct VantEngine {
    inner: TelexEngine,
    text_buf: CString,
    raw_buf: CString,
}

impl VantEngine {
    fn empty_result() -> VantKeyResult {
        VantKeyResult {
            event_type: VantEventType::Passthrough,
            text: std::ptr::null(),
            text_len: 0,
            raw: std::ptr::null(),
            raw_len: 0,
            committed_char: 0,
        }
    }

    fn event_to_result(&mut self, event: SyllableEvent) -> VantKeyResult {
        match event {
            SyllableEvent::Composing { raw, composed } => {
                self.text_buf = CString::new(composed.as_bytes()).unwrap_or_default();
                self.raw_buf = CString::new(raw.as_bytes()).unwrap_or_default();
                VantKeyResult {
                    event_type: VantEventType::Composing,
                    text: self.text_buf.as_ptr(),
                    text_len: composed.len() as u32,
                    raw: self.raw_buf.as_ptr(),
                    raw_len: raw.len() as u32,
                    committed_char: 0,
                }
            }
            SyllableEvent::Committed { text, committed_by } => {
                let text_len = text.len() as u32;
                self.text_buf = CString::new(text.as_bytes()).unwrap_or_default();
                self.raw_buf = CString::default();
                VantKeyResult {
                    event_type: VantEventType::Committed,
                    text: self.text_buf.as_ptr(),
                    text_len,
                    raw: self.raw_buf.as_ptr(),
                    raw_len: 0,
                    committed_char: committed_by.map(|c| c as u32).unwrap_or(0),
                }
            }
            SyllableEvent::Reset => {
                self.text_buf = CString::default();
                self.raw_buf = CString::default();
                VantKeyResult {
                    event_type: VantEventType::Reset,
                    text: self.text_buf.as_ptr(),
                    text_len: 0,
                    raw: self.raw_buf.as_ptr(),
                    raw_len: 0,
                    committed_char: 0,
                }
            }
            SyllableEvent::Passthrough => {
                self.text_buf = CString::default();
                self.raw_buf = CString::default();
                VantKeyResult {
                    event_type: VantEventType::Passthrough,
                    text: self.text_buf.as_ptr(),
                    text_len: 0,
                    raw: self.raw_buf.as_ptr(),
                    raw_len: 0,
                    committed_char: 0,
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// FFI functions
// ---------------------------------------------------------------------------

/// Create a new TelexEngine. Returns a pointer that must be freed with
/// `vant_engine_destroy`.
#[no_mangle]
pub extern "C" fn vant_engine_create() -> *mut VantEngine {
    let engine = VantEngine {
        inner: TelexEngine::new(),
        text_buf: CString::default(),
        raw_buf: CString::default(),
    };
    Box::into_raw(Box::new(engine))
}

/// Free a TelexEngine created by `vant_engine_create`.
#[no_mangle]
pub extern "C" fn vant_engine_destroy(engine: *mut VantEngine) {
    if !engine.is_null() {
        unsafe {
            drop(Box::from_raw(engine));
        }
    }
}

/// Process a single keystroke.
///
/// - `key_char`: Unicode codepoint of the key (ignored for backspace/escape)
/// - `is_backspace`: true if the key is backspace
/// - `is_escape`: true if the key is escape
///
/// Returns a `VantKeyResult` whose string pointers are valid until the next
/// call on the same engine.
#[no_mangle]
pub extern "C" fn vant_engine_process_key(
    engine: *mut VantEngine,
    key_char: u32,
    is_backspace: bool,
    is_escape: bool,
) -> VantKeyResult {
    let Some(engine) = (unsafe { engine.as_mut() }) else {
        return VantEngine::empty_result();
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let ch = char::from_u32(key_char).unwrap_or('\0');
        engine.inner.process_key(ch, is_backspace, is_escape)
    }));
    match result {
        Ok(event) => engine.event_to_result(event),
        Err(_) => VantEngine::empty_result(),
    }
}

/// Force-commit the current composition (e.g. on focus change).
#[no_mangle]
pub extern "C" fn vant_engine_force_commit(engine: *mut VantEngine) -> VantKeyResult {
    let Some(engine) = (unsafe { engine.as_mut() }) else {
        return VantEngine::empty_result();
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        engine.inner.force_commit()
    }));
    match result {
        Ok(event) => engine.event_to_result(event),
        Err(_) => VantEngine::empty_result(),
    }
}

/// Reset the engine, cancelling any active composition.
#[no_mangle]
pub extern "C" fn vant_engine_reset(engine: *mut VantEngine) -> VantKeyResult {
    let Some(engine) = (unsafe { engine.as_mut() }) else {
        return VantEngine::empty_result();
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        engine.inner.reset()
    }));
    match result {
        Ok(event) => engine.event_to_result(event),
        Err(_) => VantEngine::empty_result(),
    }
}

/// Returns 1 if the engine has an active composition, 0 otherwise.
#[no_mangle]
pub extern "C" fn vant_engine_is_composing(engine: *const VantEngine) -> i32 {
    let Some(engine) = (unsafe { engine.as_ref() }) else {
        return 0;
    };
    if engine.inner.is_composing() { 1 } else { 0 }
}
