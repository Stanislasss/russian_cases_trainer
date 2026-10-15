//! The desktop app's speech pack: native Piper and one voice, downloaded on
//! request from the URLs in `backend/speech-pack.json`, then run as a child
//! process. Piper's own espeak-ng data ignores stress marks, so the pack ships
//! the newer data from piper-tts, passed with `--espeak_data`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use application::ports::BoxError;
use application::speech::{PackStatus, SpeechPack, SpeechSynthesizer};
use async_trait::async_trait;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

/// The one voice the pack contains.
pub const VOICE: &str = "ru_RU-irina-medium";

/// How long one word may take before Piper is considered stuck.
const SYNTHESIS_TIMEOUT: Duration = Duration::from_secs(30);

/// Marks a complete installation; written last.
const INSTALLED: &str = "installed";

/// Where the pack's parts are published (`backend/speech-pack.json`).
#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    /// Piper and its libraries per platform (`<os>-<arch>`, as in Rust's
    /// `std::env::consts`), plus the espeak-ng data.
    pub piper: HashMap<String, Download>,
    pub voice: Download,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Download {
    pub url: String,
    /// Hex SHA-256 of the zip file.
    pub sha256: String,
    /// Bytes, for the progress bar.
    pub size: u64,
}

impl Manifest {
    fn piper_here(&self) -> Option<&Download> {
        let platform = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
        self.piper.get(&platform)
    }
}

/// Both the pack (install/remove) and the synthesizer that uses it.
#[derive(Clone)]
pub struct DesktopSpeech {
    inner: Arc<Inner>,
}

struct Inner {
    dir: PathBuf,
    manifest: Manifest,
    client: reqwest::Client,
    status: Mutex<PackStatus>,
    piper: tokio::sync::Mutex<Option<Piper>>,
}

/// A running `piper --json-input`: one JSON line in, one output path out.
struct Piper {
    _child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
    requests: u64,
}

impl DesktopSpeech {
    /// The pack lives in `dir` (created when installed).
    pub fn new(dir: impl Into<PathBuf>, manifest: Manifest) -> Self {
        let dir = dir.into();
        let status = if dir.join(INSTALLED).is_file() {
            PackStatus::Ready
        } else {
            PackStatus::NotInstalled
        };
        Self {
            inner: Arc::new(Inner {
                dir,
                manifest,
                client: reqwest::Client::new(),
                status: Mutex::new(status),
                piper: tokio::sync::Mutex::new(None),
            }),
        }
    }
}

#[async_trait]
impl SpeechPack for DesktopSpeech {
    fn status(&self) -> PackStatus {
        self.inner.status.lock().expect("not poisoned").clone()
    }

    fn install(&self) {
        {
            let mut status = self.inner.status.lock().expect("not poisoned");
            if matches!(*status, PackStatus::Installing { .. } | PackStatus::Ready) {
                return;
            }
            *status = PackStatus::Installing { progress: 0.0 };
        }
        let inner = self.inner.clone();
        tokio::spawn(async move {
            let status = match inner.download_and_unpack().await {
                Ok(()) => PackStatus::Ready,
                Err(e) => {
                    tracing::warn!(error = %e, "installing the speech pack failed");
                    PackStatus::Failed {
                        error: e.to_string(),
                    }
                }
            };
            *inner.status.lock().expect("not poisoned") = status;
        });
    }

    async fn remove(&self) -> Result<(), BoxError> {
        if matches!(self.status(), PackStatus::Installing { .. }) {
            return Err("the speech pack is being installed".into());
        }
        self.inner.piper.lock().await.take(); // stops Piper (kill on drop)
        remove_dir(&self.inner.dir).await?;
        *self.inner.status.lock().expect("not poisoned") = PackStatus::NotInstalled;
        Ok(())
    }
}

#[async_trait]
impl SpeechSynthesizer for DesktopSpeech {
    fn voice(&self) -> &str {
        VOICE
    }

    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, BoxError> {
        if self.status() != PackStatus::Ready {
            return Err("the speech pack isn't installed".into());
        }
        let mut piper = self.inner.piper.lock().await;
        if piper.is_none() {
            *piper = Some(self.inner.start_piper()?);
        }
        let running = piper.as_mut().expect("just started");
        let result =
            tokio::time::timeout(SYNTHESIS_TIMEOUT, self.inner.synthesize_with(running, text))
                .await
                .unwrap_or_else(|_| Err("Piper took too long".into()));
        if result.is_err() {
            *piper = None; // restart it next time
        }
        result
    }
}

impl Inner {
    fn start_piper(&self) -> Result<Piper, BoxError> {
        let exe = self
            .dir
            .join(if cfg!(windows) { "piper.exe" } else { "piper" });
        let mut command = Command::new(&exe);
        command
            .arg("--model")
            .arg(self.dir.join(format!("{VOICE}.onnx")))
            .arg("--espeak_data")
            .arg(self.dir.join("espeak-ng-data"))
            .arg("--json-input")
            .arg("--quiet")
            .current_dir(&self.dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        #[cfg(windows)]
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: no console flashing up
        let mut child = command
            .spawn()
            .map_err(|e| format!("starting {}: {e}", exe.display()))?;
        let stdin = child.stdin.take().expect("piped");
        let stdout = BufReader::new(child.stdout.take().expect("piped")).lines();
        Ok(Piper {
            _child: child,
            stdin,
            stdout,
            requests: 0,
        })
    }

    async fn synthesize_with(&self, piper: &mut Piper, text: &str) -> Result<Vec<u8>, BoxError> {
        piper.requests += 1;
        let out = self.dir.join("out");
        tokio::fs::create_dir_all(&out).await?;
        let file = out.join(format!("{}.wav", piper.requests));
        let mut line = serde_json::to_vec(&serde_json::json!({
            "text": text,
            "output_file": file,
        }))?;
        line.push(b'\n');
        piper.stdin.write_all(&line).await?;
        piper.stdin.flush().await?;
        // Piper answers with the path once the file is written.
        piper.stdout.next_line().await?.ok_or("Piper stopped")?;
        let audio = tokio::fs::read(&file).await?;
        let _ = tokio::fs::remove_file(&file).await;
        if !audio.starts_with(b"RIFF") {
            return Err("Piper didn't write WAV audio".into());
        }
        Ok(audio)
    }

    async fn download_and_unpack(&self) -> Result<(), BoxError> {
        let piper = self
            .manifest
            .piper_here()
            .ok_or("there's no speech pack for this system yet")?;
        let parts = [piper, &self.manifest.voice];
        let total: u64 = parts.iter().map(|d| d.size).sum::<u64>().max(1);

        // Unpacked next to the final place, then swapped in.
        let staging = self.dir.with_extension("new");
        remove_dir(&staging).await?;
        tokio::fs::create_dir_all(&staging).await?;
        let mut done = 0;
        for part in parts {
            let archive = staging.join("download.zip");
            self.download(part, &archive, &mut done, total).await?;
            let into = staging.clone();
            tokio::task::spawn_blocking(move || unzip(&archive, &into)).await??;
        }
        tokio::fs::remove_file(staging.join("download.zip")).await?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let exe = staging.join("piper");
            tokio::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755)).await?;
        }

        remove_dir(&self.dir).await?;
        tokio::fs::rename(&staging, &self.dir).await?;
        tokio::fs::write(self.dir.join(INSTALLED), VOICE).await?;
        Ok(())
    }

    /// Downloads `part` to `path`, checking its hash and reporting progress.
    async fn download(
        &self,
        part: &Download,
        path: &Path,
        done: &mut u64,
        total: u64,
    ) -> Result<(), BoxError> {
        let mut response = self
            .client
            .get(&part.url)
            .send()
            .await?
            .error_for_status()?;
        let mut file = tokio::fs::File::create(path).await?;
        let mut hash = Sha256::new();
        while let Some(chunk) = response.chunk().await? {
            hash.update(&chunk);
            file.write_all(&chunk).await?;
            *done += chunk.len() as u64;
            *self.status.lock().expect("not poisoned") = PackStatus::Installing {
                progress: (*done as f32 / total as f32).min(1.0),
            };
        }
        file.flush().await?;
        let actual: String = hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if !actual.eq_ignore_ascii_case(&part.sha256) {
            return Err(format!("{} is damaged (checksum mismatch)", part.url).into());
        }
        Ok(())
    }
}

/// Extracts `archive` into `dir`; entries can't escape it.
fn unzip(archive: &Path, dir: &Path) -> Result<(), BoxError> {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(archive)?)?;
    zip.extract(dir)?;
    Ok(())
}

async fn remove_dir(dir: &Path) -> std::io::Result<()> {
    match tokio::fs::remove_dir_all(dir).await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::time::Instant;

    use tokio::io::AsyncReadExt;
    use tokio::net::TcpListener;

    use super::*;

    fn zip_of(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, content) in files {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(content).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    fn sha256(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// Serves each of `files` (path → body) over HTTP; returns the base URL.
    async fn serve(files: HashMap<String, Vec<u8>>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = [0; 1024];
                let n = socket.read(&mut request).await.unwrap();
                let request = String::from_utf8_lossy(&request[..n]);
                let path = request.split_whitespace().nth(1).unwrap_or("/");
                let body = files.get(path).cloned().unwrap_or_default();
                let head = format!(
                    "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                    body.len()
                );
                socket.write_all(head.as_bytes()).await.unwrap();
                socket.write_all(&body).await.unwrap();
            }
        });
        base
    }

    /// A stand-in for Piper speaking its protocol: writes "RIFF" + the request
    /// line to the requested file, then prints the path.
    const FAKE_PIPER: &[u8] = br#"#!/bin/sh
while IFS= read -r line; do
  out=$(printf '%s' "$line" | sed 's/.*"output_file":"\([^"]*\)".*/\1/')
  printf 'RIFF%s' "$line" > "$out"
  echo "$out"
done
"#;

    async fn pack_from(
        piper_zip: Vec<u8>,
        sha: Option<String>,
    ) -> (DesktopSpeech, tempfile::TempDir) {
        let voice_zip = zip_of(&[(&format!("{VOICE}.onnx"), b"model")]);
        let manifest = Manifest {
            piper: HashMap::new(),
            voice: Download {
                url: String::new(),
                sha256: sha256(&voice_zip),
                size: voice_zip.len() as u64,
            },
        };
        let base = serve(HashMap::from([
            ("/piper.zip".to_owned(), piper_zip.clone()),
            ("/voice.zip".to_owned(), voice_zip),
        ]))
        .await;
        let platform = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
        let manifest = Manifest {
            piper: HashMap::from([(
                platform,
                Download {
                    url: format!("{base}/piper.zip"),
                    sha256: sha.unwrap_or_else(|| sha256(&piper_zip)),
                    size: piper_zip.len() as u64,
                },
            )]),
            voice: Download {
                url: format!("{base}/voice.zip"),
                ..manifest.voice
            },
        };
        let dir = tempfile::tempdir().unwrap();
        (DesktopSpeech::new(dir.path().join("pack"), manifest), dir)
    }

    async fn settled(pack: &DesktopSpeech) -> PackStatus {
        let start = Instant::now();
        loop {
            let status = pack.status();
            if !matches!(status, PackStatus::Installing { .. }) {
                return status;
            }
            assert!(
                start.elapsed() < Duration::from_secs(10),
                "still installing"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn installs_then_speaks_then_removes() {
        let (pack, _dir) = pack_from(zip_of(&[("piper", FAKE_PIPER)]), None).await;
        assert_eq!(pack.status(), PackStatus::NotInstalled);
        assert!(pack.synthesize("кни́га").await.is_err());

        pack.install();
        assert_eq!(settled(&pack).await, PackStatus::Ready);
        let audio = pack.synthesize("кни́га").await.unwrap();
        assert!(audio.starts_with(b"RIFF"));
        assert!(String::from_utf8_lossy(&audio).contains("кни́га"));
        // the same process answers the next request
        assert!(pack.synthesize("стол").await.is_ok());

        pack.remove().await.unwrap();
        assert_eq!(pack.status(), PackStatus::NotInstalled);
        assert!(pack.synthesize("кни́га").await.is_err());
    }

    #[tokio::test]
    async fn a_damaged_download_is_not_installed() {
        let (pack, dir) = pack_from(zip_of(&[("piper", b"x")]), Some("00".repeat(32))).await;
        pack.install();
        let PackStatus::Failed { error } = settled(&pack).await else {
            panic!("installed a damaged pack");
        };
        assert!(error.contains("checksum"), "{error}");
        assert!(!dir.path().join("pack").exists());
    }

    #[tokio::test]
    async fn an_installed_pack_is_found_again() {
        let (pack, dir) = pack_from(zip_of(&[("piper", b"x")]), None).await;
        pack.install();
        assert_eq!(settled(&pack).await, PackStatus::Ready);
        let manifest = pack.inner.manifest.clone();
        let again = DesktopSpeech::new(dir.path().join("pack"), manifest);
        assert_eq!(again.status(), PackStatus::Ready);
    }
}
