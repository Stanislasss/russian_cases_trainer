//! Pronouncing Russian words. Audio is synthesized once per voice and text,
//! then served from a cache.

use std::sync::Arc;

use async_trait::async_trait;
use domain::stress;
use sha2::{Digest, Sha256};

use crate::ports::BoxError;

/// Words and short example sentences only: this isn't a general text-to-speech proxy.
pub const MAX_CHARS: usize = 100;

/// Punctuation allowed in example sentences («У меня́ нет вре́мени.»).
const PUNCTUATION: &[char] = &['.', ',', '!', '?', ':', ';', '—', '–', '«', '»', '(', ')'];

#[derive(Debug, thiserror::Error)]
pub enum SpeechError {
    #[error("only short Russian words and sentences can be pronounced")]
    InvalidText,
    #[error("speech synthesis is unavailable")]
    Unavailable(#[source] BoxError),
}

#[async_trait]
pub trait SpeechSynthesizer: Send + Sync {
    /// Identifies the voice, so that changing it doesn't serve old audio.
    fn voice(&self) -> &str;

    /// WAV audio of `text`. Stress marks (U+0301) are honoured.
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, BoxError>;
}

#[async_trait]
pub trait AudioCache: Send + Sync {
    async fn get(&self, key: &str) -> Option<Vec<u8>>;
    async fn put(&self, key: &str, audio: &[u8]) -> Result<(), BoxError>;
}

#[derive(Clone)]
pub struct SpeechService {
    synthesizer: Arc<dyn SpeechSynthesizer>,
    cache: Arc<dyn AudioCache>,
}

impl SpeechService {
    pub fn new(synthesizer: Arc<dyn SpeechSynthesizer>, cache: Arc<dyn AudioCache>) -> Self {
        Self { synthesizer, cache }
    }

    /// WAV audio of a Russian word or short sentence, ideally stress-marked (кни́гой).
    pub async fn say(&self, text: &str) -> Result<Vec<u8>, SpeechError> {
        let text = text.trim();
        let words: String = text.chars().filter(|c| !PUNCTUATION.contains(c)).collect();
        if text.chars().count() > MAX_CHARS || !stress::is_russian(&words) {
            return Err(SpeechError::InvalidText);
        }

        let key = cache_key(self.synthesizer.voice(), text);
        if let Some(audio) = self.cache.get(&key).await {
            return Ok(audio);
        }
        let audio = self
            .synthesizer
            .synthesize(text)
            .await
            .map_err(SpeechError::Unavailable)?;
        // The cache only saves work; a failed write doesn't fail the request.
        let _ = self.cache.put(&key, &audio).await;
        Ok(audio)
    }
}

/// Hex SHA-256 of voice and text: safe as a file name, changes with either.
fn cache_key(voice: &str, text: &str) -> String {
    Sha256::digest(format!("{voice}\n{text}"))
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[derive(Default)]
    struct FakeSynthesizer {
        calls: AtomicUsize,
        down: bool,
    }

    #[async_trait]
    impl SpeechSynthesizer for FakeSynthesizer {
        fn voice(&self) -> &str {
            "test-voice"
        }

        async fn synthesize(&self, text: &str) -> Result<Vec<u8>, BoxError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            if self.down {
                return Err("connection refused".into());
            }
            Ok(format!("RIFF:{text}").into_bytes())
        }
    }

    #[derive(Default)]
    struct FakeCache(Mutex<HashMap<String, Vec<u8>>>);

    #[async_trait]
    impl AudioCache for FakeCache {
        async fn get(&self, key: &str) -> Option<Vec<u8>> {
            self.0.lock().unwrap().get(key).cloned()
        }

        async fn put(&self, key: &str, audio: &[u8]) -> Result<(), BoxError> {
            self.0
                .lock()
                .unwrap()
                .insert(key.to_owned(), audio.to_vec());
            Ok(())
        }
    }

    fn service(down: bool) -> (SpeechService, Arc<FakeSynthesizer>) {
        let synthesizer = Arc::new(FakeSynthesizer {
            down,
            ..Default::default()
        });
        let service = SpeechService::new(synthesizer.clone(), Arc::new(FakeCache::default()));
        (service, synthesizer)
    }

    #[tokio::test]
    async fn synthesizes_once_then_serves_from_cache() {
        let (speech, synthesizer) = service(false);
        let first = speech.say(" кни\u{301}гой ").await.unwrap();
        let second = speech.say("кни\u{301}гой").await.unwrap();

        assert_eq!(first, "RIFF:кни\u{301}гой".as_bytes()); // trimmed
        assert_eq!(first, second);
        assert_eq!(synthesizer.calls.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn stress_is_part_of_the_key() {
        let (speech, synthesizer) = service(false);
        speech.say("ру\u{301}ки").await.unwrap();
        speech.say("руки\u{301}").await.unwrap();
        assert_eq!(synthesizer.calls.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn only_short_russian_text_is_spoken() {
        let (speech, synthesizer) = service(false);
        for text in [
            "hello",
            "",
            "книга!!1",
            "кни́га, book",
            "?!",
            &"а".repeat(MAX_CHARS + 1),
        ] {
            assert!(
                matches!(speech.say(text).await, Err(SpeechError::InvalidText)),
                "{text}"
            );
        }
        assert_eq!(synthesizer.calls.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn example_sentences_can_be_spoken() {
        let (speech, _) = service(false);
        assert!(speech.say("У меня́ нет вре́мени.").await.is_ok());
        assert!(speech.say("Москва́ — столи́ца Росси́и.").await.is_ok());
    }

    #[tokio::test]
    async fn synthesizer_failures_are_reported() {
        let (speech, _) = service(true);
        assert!(matches!(
            speech.say("книга").await,
            Err(SpeechError::Unavailable(_))
        ));
    }
}
