use crate::event::SyllableEvent;
use crate::tone;

/// Stateful Telex input engine. Wraps the `vi` crate's stateless
/// `transform_buffer` to process keystrokes one at a time and emit
/// `SyllableEvent`s for the frontend.
pub struct TelexEngine {
    raw_buffer: Vec<char>,
    last_composed: String,
}

/// Characters that finalize the current composition.
const COMMIT_TRIGGERS: &[char] = &[
    ' ', '\n', '\t', '.', ',', ';', ':', '!', '?', '\'', '"', '(', ')', '-', '/', '\\', '@', '#',
    '$', '%', '^', '&', '*', '+', '=', '[', ']', '{', '}', '|', '<', '>', '~',
];

impl TelexEngine {
    pub fn new() -> Self {
        Self {
            raw_buffer: Vec::new(),
            last_composed: String::new(),
        }
    }

    /// Process a single keystroke and return the resulting event.
    pub fn process_key(&mut self, ch: char, is_backspace: bool, is_escape: bool) -> SyllableEvent {
        if is_escape {
            return if self.raw_buffer.is_empty() {
                SyllableEvent::Passthrough
            } else {
                self.clear();
                SyllableEvent::Reset
            };
        }

        if is_backspace {
            return if self.raw_buffer.is_empty() {
                SyllableEvent::Passthrough
            } else {
                self.raw_buffer.pop();
                if self.raw_buffer.is_empty() {
                    self.last_composed.clear();
                    SyllableEvent::Reset
                } else {
                    self.recompose();
                    SyllableEvent::Composing {
                        raw: self.raw_buffer_string(),
                        composed: self.last_composed.clone(),
                    }
                }
            };
        }

        if Self::is_commit_trigger(ch) {
            return if self.raw_buffer.is_empty() {
                SyllableEvent::Passthrough
            } else {
                let text = self.last_composed.clone();
                self.clear();
                SyllableEvent::Committed {
                    text,
                    committed_by: Some(ch),
                }
            };
        }

        if Self::is_telex_input(ch) {
            self.raw_buffer.push(ch);
            self.recompose();
            return SyllableEvent::Composing {
                raw: self.raw_buffer_string(),
                composed: self.last_composed.clone(),
            };
        }

        // Non-telex input (digits, symbols) while composing → commit first
        if !self.raw_buffer.is_empty() {
            let text = self.last_composed.clone();
            self.clear();
            return SyllableEvent::Committed {
                text,
                committed_by: Some(ch),
            };
        }

        SyllableEvent::Passthrough
    }

    /// Commit the current composition programmatically (e.g. on focus change).
    pub fn force_commit(&mut self) -> SyllableEvent {
        if self.raw_buffer.is_empty() {
            return SyllableEvent::Passthrough;
        }
        let text = self.last_composed.clone();
        self.clear();
        SyllableEvent::Committed {
            text,
            committed_by: None,
        }
    }

    /// Cancel the current composition.
    pub fn reset(&mut self) -> SyllableEvent {
        if self.raw_buffer.is_empty() {
            return SyllableEvent::Passthrough;
        }
        self.clear();
        SyllableEvent::Reset
    }

    pub fn is_composing(&self) -> bool {
        !self.raw_buffer.is_empty()
    }

    pub fn composed_text(&self) -> &str {
        &self.last_composed
    }

    pub fn raw_buffer(&self) -> String {
        self.raw_buffer_string()
    }

    // -- private --

    fn recompose(&mut self) {
        // The vi crate can panic on certain inputs (e.g. vowel + 'd') due to an
        // unguarded .expect() in vi::processor::modify_letter. Catch any panic
        // and fall back to the raw buffer so composition stays alive.
        let raw: Vec<char> = self.raw_buffer.clone();
        let result = std::panic::catch_unwind(|| {
            let mut out = String::new();
            vi::telex::transform_buffer(raw.iter().copied(), &mut out);
            out
        });
        self.last_composed = match result {
            Ok(mut composed) => {
                tone::fix_incomplete_horn(&mut composed);
                tone::relocate_tone(&mut composed);
                composed
            }
            Err(_) => self.raw_buffer.iter().collect(),
        };
    }

    fn clear(&mut self) {
        self.raw_buffer.clear();
        self.last_composed.clear();
    }

    fn raw_buffer_string(&self) -> String {
        self.raw_buffer.iter().collect()
    }

    fn is_commit_trigger(ch: char) -> bool {
        COMMIT_TRIGGERS.contains(&ch)
    }

    fn is_telex_input(ch: char) -> bool {
        ch.is_ascii_alphabetic()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: feed a string into a fresh engine, return the last event.
    fn type_keys(input: &str) -> SyllableEvent {
        let mut engine = TelexEngine::new();
        let mut last = SyllableEvent::Passthrough;
        for ch in input.chars() {
            last = engine.process_key(ch, false, false);
        }
        last
    }

    /// Helper: feed keys and return the composed text from the last Composing event.
    fn compose(input: &str) -> String {
        match type_keys(input) {
            SyllableEvent::Composing { composed, .. } => composed,
            other => panic!("expected Composing, got {:?}", other),
        }
    }

    // ===== A. Diacritics =====

    #[test]
    fn diacritic_circumflex_a() {
        assert_eq!(compose("aa"), "â");
    }

    #[test]
    fn diacritic_circumflex_e() {
        assert_eq!(compose("ee"), "ê");
    }

    #[test]
    fn diacritic_circumflex_o() {
        assert_eq!(compose("oo"), "ô");
    }

    #[test]
    fn diacritic_horn_o() {
        assert_eq!(compose("ow"), "ơ");
    }

    #[test]
    fn diacritic_horn_u() {
        assert_eq!(compose("uw"), "ư");
    }

    #[test]
    fn diacritic_dyet() {
        assert_eq!(compose("dd"), "đ");
    }

    #[test]
    fn diacritic_breve() {
        assert_eq!(compose("aw"), "ă");
    }

    // ===== B. Tones =====

    #[test]
    fn tone_sac() {
        assert_eq!(compose("as"), "á");
    }

    #[test]
    fn tone_huyen() {
        assert_eq!(compose("af"), "à");
    }

    #[test]
    fn tone_hoi() {
        assert_eq!(compose("ar"), "ả");
    }

    #[test]
    fn tone_nga() {
        assert_eq!(compose("ax"), "ã");
    }

    #[test]
    fn tone_nang() {
        assert_eq!(compose("aj"), "ạ");
    }

    #[test]
    fn tone_remove() {
        assert_eq!(compose("asz"), "a");
    }

    // ===== C. Complete syllables =====

    #[test]
    fn syllable_viet() {
        assert_eq!(compose("vieetj"), "việt");
    }

    #[test]
    fn syllable_duoc() {
        assert_eq!(compose("dduowcj"), "được");
    }

    #[test]
    fn syllable_nguoi() {
        assert_eq!(compose("nguowif"), "người");
    }

    #[test]
    fn syllable_thuong() {
        assert_eq!(compose("thuowng"), "thương");
    }

    #[test]
    fn syllable_phuong() {
        assert_eq!(compose("phuowng"), "phương");
    }

    #[test]
    fn syllable_nuoc() {
        assert_eq!(compose("nuowcs"), "nước");
    }

    #[test]
    fn syllable_plain() {
        assert_eq!(compose("nam"), "nam");
    }

    // ===== D. Deferred diacritics =====
    // The `w` modifier can be typed after subsequent consonants and the vi
    // crate will retroactively apply horn/breve to the correct vowel.

    #[test]
    fn deferred_duoc() {
        // w typed after 'c' — horn applied retroactively to 'u' and 'o'
        assert_eq!(compose("duocwj"), "dược");
    }

    #[test]
    fn deferred_tuong() {
        // w typed after 'ng' — horn applied retroactively
        assert_eq!(compose("tuongw"), "tương");
    }

    #[test]
    fn deferred_nguoi() {
        // w typed after 'i' — horn applied retroactively to 'o'
        assert_eq!(compose("nguoiw"), "ngươi");
    }

    // ===== E. Backspace =====

    #[test]
    fn backspace_mid_composition() {
        let mut engine = TelexEngine::new();
        engine.process_key('v', false, false);
        engine.process_key('i', false, false);
        engine.process_key('e', false, false);
        let event = engine.process_key('\0', true, false);
        match event {
            SyllableEvent::Composing { composed, .. } => assert_eq!(composed, "vi"),
            other => panic!("expected Composing, got {:?}", other),
        }
    }

    #[test]
    fn backspace_to_empty() {
        let mut engine = TelexEngine::new();
        engine.process_key('a', false, false);
        let event = engine.process_key('\0', true, false);
        assert_eq!(event, SyllableEvent::Reset);
        assert!(!engine.is_composing());
    }

    #[test]
    fn backspace_from_empty() {
        let mut engine = TelexEngine::new();
        let event = engine.process_key('\0', true, false);
        assert_eq!(event, SyllableEvent::Passthrough);
    }

    #[test]
    fn backspace_after_tone() {
        let mut engine = TelexEngine::new();
        // Type "as" → "á", then backspace removes the 's'
        engine.process_key('a', false, false);
        engine.process_key('s', false, false);
        let event = engine.process_key('\0', true, false);
        match event {
            SyllableEvent::Composing { composed, .. } => assert_eq!(composed, "a"),
            other => panic!("expected Composing, got {:?}", other),
        }
    }

    // ===== F. Commit triggers =====

    #[test]
    fn commit_on_space() {
        let mut engine = TelexEngine::new();
        engine.process_key('a', false, false);
        engine.process_key('s', false, false);
        let event = engine.process_key(' ', false, false);
        assert_eq!(
            event,
            SyllableEvent::Committed {
                text: "á".to_string(),
                committed_by: Some(' '),
            }
        );
        assert!(!engine.is_composing());
    }

    #[test]
    fn commit_on_period() {
        let mut engine = TelexEngine::new();
        engine.process_key('a', false, false);
        let event = engine.process_key('.', false, false);
        assert_eq!(
            event,
            SyllableEvent::Committed {
                text: "a".to_string(),
                committed_by: Some('.'),
            }
        );
    }

    #[test]
    fn commit_on_comma() {
        let mut engine = TelexEngine::new();
        engine.process_key('a', false, false);
        let event = engine.process_key(',', false, false);
        assert_eq!(
            event,
            SyllableEvent::Committed {
                text: "a".to_string(),
                committed_by: Some(','),
            }
        );
    }

    #[test]
    fn commit_on_enter() {
        let mut engine = TelexEngine::new();
        engine.process_key('a', false, false);
        let event = engine.process_key('\n', false, false);
        assert_eq!(
            event,
            SyllableEvent::Committed {
                text: "a".to_string(),
                committed_by: Some('\n'),
            }
        );
    }

    #[test]
    fn space_no_composition_passthrough() {
        let mut engine = TelexEngine::new();
        let event = engine.process_key(' ', false, false);
        assert_eq!(event, SyllableEvent::Passthrough);
    }

    // ===== G. Reset / Force commit =====

    #[test]
    fn escape_clears_composition() {
        let mut engine = TelexEngine::new();
        engine.process_key('v', false, false);
        engine.process_key('i', false, false);
        let event = engine.process_key('\0', false, true);
        assert_eq!(event, SyllableEvent::Reset);
        assert!(!engine.is_composing());
    }

    #[test]
    fn escape_empty_passthrough() {
        let mut engine = TelexEngine::new();
        let event = engine.process_key('\0', false, true);
        assert_eq!(event, SyllableEvent::Passthrough);
    }

    #[test]
    fn force_commit_active() {
        let mut engine = TelexEngine::new();
        engine.process_key('v', false, false);
        engine.process_key('i', false, false);
        let event = engine.force_commit();
        assert_eq!(
            event,
            SyllableEvent::Committed {
                text: "vi".to_string(),
                committed_by: None,
            }
        );
        assert!(!engine.is_composing());
    }

    #[test]
    fn force_commit_empty() {
        let mut engine = TelexEngine::new();
        let event = engine.force_commit();
        assert_eq!(event, SyllableEvent::Passthrough);
    }

    // ===== H. Edge cases =====

    #[test]
    fn digit_commits_first() {
        let mut engine = TelexEngine::new();
        engine.process_key('a', false, false);
        engine.process_key('s', false, false);
        let event = engine.process_key('1', false, false);
        assert_eq!(
            event,
            SyllableEvent::Committed {
                text: "á".to_string(),
                committed_by: Some('1'),
            }
        );
        assert!(!engine.is_composing());
    }

    #[test]
    fn digit_no_composition() {
        let mut engine = TelexEngine::new();
        let event = engine.process_key('1', false, false);
        assert_eq!(event, SyllableEvent::Passthrough);
    }

    #[test]
    fn uppercase_preserved() {
        assert_eq!(compose("AS"), "Á");
    }

    #[test]
    fn sequential_syllables() {
        let mut engine = TelexEngine::new();
        // First syllable: "vieetj" → "việt", commit with space
        for ch in "vieetj".chars() {
            engine.process_key(ch, false, false);
        }
        let event = engine.process_key(' ', false, false);
        assert_eq!(
            event,
            SyllableEvent::Committed {
                text: "việt".to_string(),
                committed_by: Some(' '),
            }
        );

        // Second syllable: "nam"
        for ch in "nam".chars() {
            engine.process_key(ch, false, false);
        }
        assert_eq!(engine.composed_text(), "nam");
    }

    // ===== I. Tone relocation (tone typed before final consonant) =====

    #[test]
    fn tone_before_consonant_xuat() {
        // User types tone 's' before final 't': xuaast → xuất
        assert_eq!(compose("xuaast"), "xuất");
    }

    #[test]
    fn tone_before_consonant_viet() {
        // User types tone 'j' before final 't': vieejt → việt
        assert_eq!(compose("vieejt"), "việt");
    }

    #[test]
    fn tone_before_consonant_hoang() {
        // User types tone 's' before 'ng': hoasng → hoáng
        assert_eq!(compose("hoasng"), "hoáng");
    }

    #[test]
    fn horn_before_o_truong() {
        // User types 'w' before 'o': truwong → trương
        assert_eq!(compose("truwowng"), "trương");
    }

    #[test]
    fn horn_before_o_duoc() {
        // User types 'w' before 'o': dduwocj → được
        assert_eq!(compose("dduwocj"), "được");
    }

    #[test]
    fn tone_oa_no_ending() {
        // hoa + grave → hòa (tone on 'o', no ending consonant)
        assert_eq!(compose("hoaf"), "hòa");
    }

    #[test]
    fn tone_oa_with_ending() {
        // hoan + tone → hoán (tone on 'a', has ending consonant)
        assert_eq!(compose("hoans"), "hoán");
    }

    #[test]
    fn tone_ua_no_ending() {
        // c + u + s(acute→cú) + a + r(hỏi relocates to 'u') → của
        assert_eq!(compose("cusar"), "của");
    }

    // ===== J. Vowel-before-d display regression =====
    // The engine always produced the correct text; the bug was in Swift's
    // setMarkedText using {NSNotFound,0} replacementRange. These tests
    // confirm the engine side is correct.

    #[test]
    fn vowel_then_d() {
        assert_eq!(compose("ad"), "ad");
    }

    // ===== K. vi crate panic guard (vowel + 'd') =====
    // vi 0.3.8 has an unguarded .expect() in modify_letter that panics when
    // Dyet is applied to a non-'d' letter (e.g. ['a','d'] or ['b','a','d']).
    // Our catch_unwind in recompose falls back to the raw buffer string.

    #[test]
    fn vowel_then_dd() {
        // vi panics on ['a','d','d']; raw buffer fallback produces "add"
        assert_eq!(compose("add"), "add");
    }

    #[test]
    fn bad_plain() {
        assert_eq!(compose("bad"), "bad");
    }

    // ===== Accessors =====

    #[test]
    fn composed_text_accessor() {
        let mut engine = TelexEngine::new();
        engine.process_key('a', false, false);
        engine.process_key('s', false, false);
        assert_eq!(engine.composed_text(), "á");
    }

    #[test]
    fn raw_buffer_accessor() {
        let mut engine = TelexEngine::new();
        engine.process_key('a', false, false);
        engine.process_key('s', false, false);
        assert_eq!(engine.raw_buffer(), "as");
    }

    #[test]
    fn is_composing_accessor() {
        let mut engine = TelexEngine::new();
        assert!(!engine.is_composing());
        engine.process_key('a', false, false);
        assert!(engine.is_composing());
    }
}
