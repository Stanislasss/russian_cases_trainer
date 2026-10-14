//! Stress marks. Forms are stored with U+0301 COMBINING ACUTE ACCENT after the
//! stressed vowel (кни́га); the OpenRussian dumps use an apostrophe instead (кни'га).

pub const STRESS_MARK: char = '\u{0301}';

/// Converts OpenRussian's apostrophe notation to the combining accent.
pub fn from_apostrophes(word: &str) -> String {
    word.replace('\'', "\u{0301}")
}

/// Removes stress marks, leaving the plain spelling.
pub fn strip(word: &str) -> String {
    word.chars().filter(|&c| c != STRESS_MARK).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apostrophes_become_combining_accents() {
        assert_eq!(from_apostrophes("кни'га"), "кни\u{0301}га");
    }

    #[test]
    fn strip_removes_only_stress_marks() {
        assert_eq!(strip("кни\u{0301}га"), "книга");
        assert_eq!(strip("ёж"), "ёж");
    }
}
