/// Post-processing tone relocation for Vietnamese text.
///
/// The `vi` crate places tone marks at the moment the tone key is processed.
/// If the user types the tone before the final consonant, the tone may land
/// on the wrong vowel. This module re-evaluates tone placement after the full
/// composed text is known and moves the tone if necessary.
use vi::processor::{add_tone_char, ToneMark};
use vi::util::{clean_char, remove_tone_mark};

/// Vietnamese initial consonant clusters, ordered longest-first for greedy matching.
const INITIAL_CONSONANTS: &[&str] = &[
    "ngh", "ng", "nh", "gh", "gi", "qu", "ph", "th", "tr", "ch", "kh", "b", "c", "d", "đ", "g",
    "h", "k", "l", "m", "n", "p", "q", "r", "s", "t", "v", "x",
];

/// Vietnamese final consonant clusters, ordered longest-first.
const FINAL_CONSONANTS: &[&str] = &["ng", "nh", "ch", "c", "m", "n", "p", "t"];

fn is_vowel_char(ch: char) -> bool {
    vi::util::is_vowel(ch)
}

/// Normalize a Vietnamese character to its base form (strip tones and diacritics).
fn base_char(ch: char) -> char {
    clean_char(ch).to_ascii_lowercase()
}

/// Check if a character has circumflex (â, ê, ô and their toned variants).
fn has_circumflex(ch: char) -> bool {
    let no_tone = remove_tone_mark(ch).to_ascii_lowercase();
    no_tone == 'â' || no_tone == 'ê' || no_tone == 'ô'
}

/// Check if a character has horn (ơ, ư and their toned variants).
fn has_horn(ch: char) -> bool {
    let no_tone = remove_tone_mark(ch).to_ascii_lowercase();
    no_tone == 'ơ' || no_tone == 'ư'
}

#[derive(Debug)]
struct SyllableComponents {
    initial: String,
    vowels: Vec<char>,
    final_cons: String,
}

/// Extract the tone mark from a character, if any.
fn extract_tone_from_char(ch: char) -> Option<ToneMark> {
    let no_tone = remove_tone_mark(ch);
    if no_tone == ch {
        // No change means no tone (or it's already base).
        // Double-check: clean_char removes diacritics too, we need just tone.
        // remove_tone_mark strips tone but keeps diacritics (â stays â).
        return None;
    }
    // Determine which tone was removed by trying each
    let lc = ch.to_lowercase().next().unwrap();
    let base = remove_tone_mark(lc);
    for (tone, map_fn) in [
        (ToneMark::Acute, 'á'),
        (ToneMark::Grave, 'à'),
        (ToneMark::HookAbove, 'ả'),
        (ToneMark::Tilde, 'ã'),
        (ToneMark::Underdot, 'ạ'),
    ] {
        let _ = map_fn; // just for illustration
        let test = add_tone_char(base, &tone);
        if test == lc {
            return Some(tone);
        }
    }
    None
}

/// Parse a composed Vietnamese string into syllable components.
///
/// Uses case-insensitive matching for consonant detection.
fn parse_syllable(text: &str) -> Option<SyllableComponents> {
    if text.is_empty() {
        return None;
    }

    let chars: Vec<char> = text.chars().collect();
    let lower: String = chars.iter().map(|c| c.to_ascii_lowercase()).collect();

    // Find initial consonant (greedy, longest match)
    // Note: use chars().count() not .len() — .len() is byte count which differs for "đ"
    let mut initial_len = 0;
    for cons in INITIAL_CONSONANTS {
        if lower.starts_with(cons) {
            let cons_chars = cons.chars().count();
            // Special case: "gi" — if followed by a vowel that is not 'i' alone,
            // treat "gi" as consonant. But "gin" → "g" + "in".
            // The vi crate's rule: "gi" is consonant unless the word is exactly "gi" or "gin".
            if *cons == "gi" {
                let rest: String = chars[cons_chars..].iter().collect();
                let rest_lower: String = rest.chars().map(|c| c.to_ascii_lowercase()).collect();
                // If nothing follows "gi", or only non-vowels follow, don't treat as cluster
                if rest.is_empty()
                    || (rest_lower == "n")
                    || (!rest.is_empty() && !is_vowel_char(rest.chars().next().unwrap()))
                {
                    // Just 'g' is the initial consonant
                    initial_len = 1;
                    break;
                }
            }
            initial_len = cons_chars;
            break;
        }
    }

    // If no known consonant matched but first char is not a vowel, take it as initial
    if initial_len == 0 && !chars.is_empty() && !is_vowel_char(chars[0]) {
        initial_len = 1;
    }

    let initial: String = chars[..initial_len].iter().collect();

    // Find where vowels end and final consonant begins.
    // Scan from initial_len: take vowels, then remaining is final consonant.
    let mut vowel_end = initial_len;
    for i in initial_len..chars.len() {
        if is_vowel_char(chars[i]) {
            vowel_end = i + 1;
        } else {
            break;
        }
    }

    let vowels: Vec<char> = chars[initial_len..vowel_end].to_vec();
    let final_cons: String = chars[vowel_end..].iter().collect();

    // Validate final consonant — must be a known Vietnamese final consonant
    if !final_cons.is_empty() {
        let fc_lower = final_cons.to_ascii_lowercase();
        if !FINAL_CONSONANTS.iter().any(|c| *c == fc_lower) {
            // Not a valid final consonant — might be mid-word junk; bail out
            return None;
        }
    }

    if vowels.is_empty() {
        return None;
    }

    Some(SyllableComponents {
        initial,
        vowels,
        final_cons,
    })
}

/// Calculate the correct tone position in a vowel cluster.
///
/// Ported from xkey's VowelSequenceValidator.calculateTonePosition().
fn calculate_tone_position(vowels: &[char], has_ending_consonant: bool, initial: &str) -> usize {
    let count = vowels.len();

    // Single vowel
    if count == 1 {
        return 0;
    }

    // Priority 1: Circumflex vowels (â, ê, ô)
    for (i, &v) in vowels.iter().enumerate() {
        if has_circumflex(v) {
            return i;
        }
    }

    // Priority 2: Horn vowels (ơ, ư)
    for (i, &v) in vowels.iter().enumerate() {
        if has_horn(v) {
            // Special case: ươ → tone on ơ (second vowel)
            if count >= 2 && i == 0 && base_char(v) == 'u' {
                let second_base = base_char(vowels[1]);
                if second_base == 'o' && has_horn(vowels[1]) {
                    return 1; // ươ, ươi, ươu → tone on ơ
                }
            }
            return i;
        }
    }

    // Triple vowels: tone on middle
    if count >= 3 {
        return 1;
    }

    // Double vowels: complex rules
    if count == 2 {
        let v0 = base_char(vowels[0]);
        let v1 = base_char(vowels[1]);

        // oi, ai, ui → first vowel
        if (v0 == 'o' || v0 == 'a' || v0 == 'u') && v1 == 'i' {
            return 0;
        }

        // ay without ending consonant → first vowel
        if v0 == 'a' && v1 == 'y' && !has_ending_consonant {
            return 0;
        }

        // oo → second vowel
        if v0 == 'o' && v1 == 'o' {
            return 1;
        }

        // ưu → first vowel
        if has_horn(vowels[0]) && base_char(vowels[0]) == 'u' && v1 == 'u' {
            return 0;
        }

        // ua without ending consonant → first vowel
        if v0 == 'u' && v1 == 'a' && !has_ending_consonant {
            return 0;
        }

        // ia, iu, io, ya: after "gi" → second vowel, otherwise → first
        if (v0 == 'i' || v0 == 'y') && (v1 == 'a' || v1 == 'u' || v1 == 'o') {
            if initial.to_ascii_lowercase() == "gi" {
                return 1;
            }
            return 0;
        }

        // General rule (from xkey's "terminated" concept):
        // Has ending consonant → second vowel (position 1)
        // No ending consonant → first vowel (position 0)
        if has_ending_consonant {
            return 1;
        } else {
            return 0;
        }
    }

    0
}

/// Fix incomplete horn pairs: ư + plain o → ư + ơ.
///
/// When the user types 'w' before 'o' (e.g., "truwong"), the vi crate applies
/// horn only to 'u' → 'ư', then 'o' is appended plain. In Vietnamese, "ưo"
/// in a vowel cluster should always be "ươ".
pub fn fix_incomplete_horn(text: &mut String) {
    let Some(syllable) = parse_syllable(text) else {
        return;
    };

    let vowels = &syllable.vowels;
    let mut fixed_vowels: Vec<char> = vowels.clone();
    let mut changed = false;

    for i in 0..vowels.len().saturating_sub(1) {
        let cur_no_tone = remove_tone_mark(vowels[i]).to_ascii_lowercase();
        let next_no_tone = remove_tone_mark(vowels[i + 1]).to_ascii_lowercase();

        // ư followed by plain o → convert o to ơ (preserving any tone)
        if cur_no_tone == 'ư' && next_no_tone == 'o' {
            let tone_on_next = extract_tone_from_char(vowels[i + 1]);
            let is_upper = vowels[i + 1].is_uppercase();
            let mut new_ch = if is_upper { 'Ơ' } else { 'ơ' };
            if let Some(t) = tone_on_next {
                new_ch = add_tone_char(new_ch, &t);
            }
            fixed_vowels[i + 1] = new_ch;
            changed = true;
        }
    }

    if !changed {
        return;
    }

    let mut result = syllable.initial.clone();
    for v in &fixed_vowels {
        result.push(*v);
    }
    result.push_str(&syllable.final_cons);
    *text = result;
}

/// Relocate the tone mark in a composed Vietnamese string if it's on the wrong vowel.
pub fn relocate_tone(text: &mut String) {
    let Some(syllable) = parse_syllable(text) else {
        return;
    };

    // Find which vowel currently has the tone
    let mut current_tone_idx: Option<usize> = None;
    let mut tone: Option<ToneMark> = None;
    for (i, &v) in syllable.vowels.iter().enumerate() {
        if let Some(t) = extract_tone_from_char(v) {
            current_tone_idx = Some(i);
            tone = Some(t);
            break;
        }
    }

    let Some(current_idx) = current_tone_idx else {
        return; // No tone mark found — nothing to relocate
    };
    let tone = tone.unwrap();

    let correct_idx = calculate_tone_position(
        &syllable.vowels,
        !syllable.final_cons.is_empty(),
        &syllable.initial,
    );

    if current_idx == correct_idx {
        return; // Already in the right place
    }

    // Rebuild the string with tone on the correct vowel
    let mut new_vowels = syllable.vowels.clone();

    // Remove tone from current position
    new_vowels[current_idx] = remove_tone_mark(new_vowels[current_idx]);

    // Add tone to correct position
    new_vowels[correct_idx] = add_tone_char(
        remove_tone_mark(new_vowels[correct_idx]),
        &tone,
    );

    // Reconstruct the full string
    let mut result = syllable.initial.clone();
    for v in &new_vowels {
        result.push(*v);
    }
    result.push_str(&syllable.final_cons);

    *text = result;
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- parse_syllable tests --

    #[test]
    fn parse_simple() {
        let s = parse_syllable("nam").unwrap();
        assert_eq!(s.initial, "n");
        assert_eq!(s.vowels, vec!['a']);
        assert_eq!(s.final_cons, "m");
    }

    #[test]
    fn parse_circumflex() {
        let s = parse_syllable("xuât").unwrap();
        assert_eq!(s.initial, "x");
        assert_eq!(s.vowels, vec!['u', 'â']);
        assert_eq!(s.final_cons, "t");
    }

    #[test]
    fn parse_viet() {
        let s = parse_syllable("viêt").unwrap();
        assert_eq!(s.initial, "v");
        assert_eq!(s.vowels, vec!['i', 'ê']);
        assert_eq!(s.final_cons, "t");
    }

    #[test]
    fn parse_nguoi() {
        let s = parse_syllable("người").unwrap();
        assert_eq!(s.initial, "ng");
        assert_eq!(s.vowels, vec!['ư', 'ờ', 'i']);
        assert_eq!(s.final_cons, "");
    }

    #[test]
    fn parse_gi_cluster() {
        let s = parse_syllable("giải").unwrap();
        assert_eq!(s.initial, "gi");
        assert_eq!(s.vowels, vec!['ả', 'i']);
        assert_eq!(s.final_cons, "");
    }

    #[test]
    fn parse_single_vowel() {
        let s = parse_syllable("ô").unwrap();
        assert_eq!(s.initial, "");
        assert_eq!(s.vowels, vec!['ô']);
        assert_eq!(s.final_cons, "");
    }

    // -- has_circumflex / has_horn tests --

    #[test]
    fn test_has_circumflex() {
        assert!(has_circumflex('â'));
        assert!(has_circumflex('ế'));
        assert!(has_circumflex('ộ'));
        assert!(!has_circumflex('a'));
        assert!(!has_circumflex('ă'));
        assert!(!has_circumflex('ơ'));
    }

    #[test]
    fn test_has_horn() {
        assert!(has_horn('ơ'));
        assert!(has_horn('ư'));
        assert!(has_horn('ớ'));
        assert!(has_horn('ự'));
        assert!(!has_horn('o'));
        assert!(!has_horn('ô'));
    }

    // -- tone position tests --

    #[test]
    fn tone_pos_single() {
        assert_eq!(calculate_tone_position(&['a'], false, ""), 0);
    }

    #[test]
    fn tone_pos_circumflex_priority() {
        assert_eq!(calculate_tone_position(&['i', 'ê'], true, "v"), 1);
        assert_eq!(calculate_tone_position(&['u', 'â'], true, "x"), 1);
        assert_eq!(calculate_tone_position(&['u', 'ô'], true, "m"), 1);
    }

    #[test]
    fn tone_pos_horn_priority() {
        assert_eq!(calculate_tone_position(&['ư', 'ơ'], true, "ng"), 1); // ươ → ơ
        assert_eq!(calculate_tone_position(&['ơ'], false, "m"), 0);
    }

    #[test]
    fn tone_pos_oi_ai_ui() {
        assert_eq!(calculate_tone_position(&['o', 'i'], false, ""), 0);
        assert_eq!(calculate_tone_position(&['a', 'i'], false, ""), 0);
        assert_eq!(calculate_tone_position(&['u', 'i'], false, ""), 0);
    }

    #[test]
    fn tone_pos_ua_no_ending() {
        assert_eq!(calculate_tone_position(&['u', 'a'], false, "c"), 0); // của
    }

    #[test]
    fn tone_pos_ua_with_ending() {
        assert_eq!(calculate_tone_position(&['u', 'a'], true, "q"), 1); // quán
    }

    #[test]
    fn tone_pos_oa_ending() {
        assert_eq!(calculate_tone_position(&['o', 'a'], false, "h"), 0); // hóa
        assert_eq!(calculate_tone_position(&['o', 'a'], true, "h"), 1);  // hoán
    }

    #[test]
    fn tone_pos_gi_ia() {
        // "gi" + "a" → tone on second vowel
        assert_eq!(calculate_tone_position(&['i', 'a'], false, "gi"), 1);
        // other + "ia" → tone on first vowel
        assert_eq!(calculate_tone_position(&['i', 'a'], false, "t"), 0);
    }

    // -- fix_incomplete_horn tests --

    #[test]
    fn fix_horn_uo_to_uoo() {
        let mut s = "trưong".to_string();
        fix_incomplete_horn(&mut s);
        assert_eq!(s, "trương");
    }

    #[test]
    fn fix_horn_already_correct() {
        let mut s = "trương".to_string();
        fix_incomplete_horn(&mut s);
        assert_eq!(s, "trương");
    }

    #[test]
    fn fix_horn_with_tone() {
        let mut s = "trưóng".to_string();
        fix_incomplete_horn(&mut s);
        assert_eq!(s, "trướng");
    }

    // -- relocate_tone integration tests --

    #[test]
    fn relocate_xuat_wrong() {
        let mut s = "xúât".to_string();
        relocate_tone(&mut s);
        assert_eq!(s, "xuất");
    }

    #[test]
    fn relocate_viet_wrong() {
        let mut s = "vịêt".to_string();
        relocate_tone(&mut s);
        assert_eq!(s, "việt");
    }

    #[test]
    fn relocate_already_correct() {
        let mut s = "việt".to_string();
        relocate_tone(&mut s);
        assert_eq!(s, "việt");
    }

    #[test]
    fn relocate_no_tone() {
        let mut s = "viêt".to_string();
        relocate_tone(&mut s);
        assert_eq!(s, "viêt");
    }

    #[test]
    fn relocate_hoa_no_ending() {
        let mut s = "hóa".to_string();
        relocate_tone(&mut s);
        assert_eq!(s, "hóa"); // correct: tone on 'o'
    }

    #[test]
    fn relocate_hoan() {
        let mut s = "hóan".to_string();
        relocate_tone(&mut s);
        assert_eq!(s, "hoán"); // move tone to 'a' (has ending)
    }

    #[test]
    fn relocate_thuong() {
        let mut s = "thương".to_string();
        relocate_tone(&mut s);
        assert_eq!(s, "thương"); // no tone, no-op
    }
}
