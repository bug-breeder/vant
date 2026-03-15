/// Events emitted by the TelexEngine in response to keystrokes.
///
/// The Swift frontend (Phase 2) will map these to InputMethodKit actions:
/// - Composing → update marked text (preedit)
/// - Committed → insert final text + clear marked text
/// - Reset → clear marked text
/// - Passthrough → let the system handle the key
#[derive(Debug, Clone, PartialEq)]
pub enum SyllableEvent {
    /// Syllable is being built. `raw` is the keystroke buffer, `composed` is
    /// the Vietnamese text after Telex transformation.
    Composing { raw: String, composed: String },

    /// Syllable has been finalized. `committed_by` is the trigger character
    /// (space, punctuation, etc.) or `None` for programmatic commits.
    Committed {
        text: String,
        committed_by: Option<char>,
    },

    /// Composition cancelled (e.g. Escape pressed).
    Reset,

    /// Key was not consumed — pass it through to the system.
    Passthrough,
}

impl SyllableEvent {
    pub fn is_composing(&self) -> bool {
        matches!(self, SyllableEvent::Composing { .. })
    }

    pub fn is_committed(&self) -> bool {
        matches!(self, SyllableEvent::Committed { .. })
    }
}
