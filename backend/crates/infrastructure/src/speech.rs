//! Speech adapters: Piper over HTTP, and a file cache for the audio.

use std::path::PathBuf;

use application::ports::BoxError;
use application::speech::{AudioCache, SpeechSynthesizer};
use async_trait::async_trait;

/// Piper's HTTP server (`python -m piper.http_server`), see `tts/Dockerfile`.
/// Its phonemizer (espeak-ng) honours U+0301, so stress-marked text is sent as is.
pub struct PiperSynthesizer {
    client: reqwest::Client,
    url: String,
    voice: String,
}

impl PiperSynthesizer {
    pub fn new(url: &str, voice: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: url.trim_end_matches('/').to_owned(),
            voice: voice.to_owned(),
        }
    }
}

#[async_trait]
impl SpeechSynthesizer for PiperSynthesizer {
    fn voice(&self) -> &str {
        &self.voice
    }

    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, BoxError> {
        let body = serde_json::json!({ "text": text, "voice": self.voice });
        let audio = self
            .client
            .post(format!("{}/synthesize", self.url))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&body)?)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        // Piper labels its WAV as text/html, so check the content itself.
        if !audio.starts_with(b"RIFF") {
            return Err("the TTS service didn't return WAV audio".into());
        }
        Ok(audio.to_vec())
    }
}

/// One `<key>.wav` file per cached text in `dir`.
pub struct FsAudioCache {
    dir: PathBuf,
}

impl FsAudioCache {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn path(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{key}.wav"))
    }
}

#[async_trait]
impl AudioCache for FsAudioCache {
    async fn get(&self, key: &str) -> Option<Vec<u8>> {
        tokio::fs::read(self.path(key)).await.ok()
    }

    async fn put(&self, key: &str, audio: &[u8]) -> Result<(), BoxError> {
        tokio::fs::create_dir_all(&self.dir).await?;
        // Write then rename, so a reader never sees a half-written file.
        let partial = self.dir.join(format!("{key}.part"));
        tokio::fs::write(&partial, audio).await?;
        tokio::fs::rename(&partial, self.path(key)).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cache_round_trip() {
        let dir = std::env::temp_dir().join(format!("speech-cache-{}", uuid::Uuid::new_v4()));
        let cache = FsAudioCache::new(&dir);

        assert_eq!(cache.get("abc").await, None);
        cache.put("abc", b"RIFF1234").await.unwrap();
        assert_eq!(cache.get("abc").await.as_deref(), Some(&b"RIFF1234"[..]));

        tokio::fs::remove_dir_all(dir).await.unwrap();
    }
}
