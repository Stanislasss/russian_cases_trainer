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

/// Cyrillic letters, optionally stress-marked and joined by hyphens or spaces
/// (пресс-конфере́нция). Rejects Latin look-alikes such as the o in "рoга".
pub fn is_russian(text: &str) -> bool {
    text.chars().any(char::is_alphabetic)
        && text
            .chars()
            .all(|c| matches!(c, 'а'..='я' | 'А'..='Я' | 'ё' | 'Ё' | '-' | ' ' | STRESS_MARK))
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

    #[test]
    fn russian_text() {
        assert!(is_russian("кни\u{0301}га"));
        assert!(is_russian("пресс-конференция"));
        assert!(!is_russian("рoга")); // Latin o
        assert!(!is_russian("MP3"));
        assert!(!is_russian(" - "));
    }
}
