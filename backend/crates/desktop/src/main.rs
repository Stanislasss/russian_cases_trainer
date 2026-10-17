//! The desktop app: runs the backend on 127.0.0.1 (see `server::start_local`)
//! and shows it in a window. Links to other sites open in the browser; new
//! versions are offered from the public releases repository.

// No console window next to the app on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::path::Path;

use anyhow::Context;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::{Update, UpdaterExt};

/// Tried first, so the app keeps its address, and with it what the page
/// stores in the browser (language, mute, game settings), across launches.
const PREFERRED_PORT: u16 = 47_913;

fn main() {
    tauri::Builder::default()
        // First: a second launch just brings the open window forward.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            if let Err(e) = start(&handle) {
                tracing::error!(error = ?e, "the app couldn't start");
                handle
                    .dialog()
                    .message(format!("{}\n\n{e:#}", texts().failed))
                    .kind(MessageDialogKind::Error)
                    .show(move |_| handle.exit(1));
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("the window system is available");
}

/// Starts the backend, opens the window and looks for updates.
fn start(app: &AppHandle) -> anyhow::Result<()> {
    let paths = app.path();
    let data_dir = paths.app_data_dir()?;
    log_to(&paths.app_log_dir()?)?;

    let server = tauri::async_runtime::block_on(server::start_local(server::LocalConfig {
        lexicon: paths.resolve("lexicon.sqlite", BaseDirectory::Resource)?,
        static_dir: Some(paths.resolve("dist", BaseDirectory::Resource)?),
        bind: free_address(&data_dir),
        data_dir,
        origin: None,
    }))?;
    // The version in the address: a new version is a page the webview has never
    // cached, so an update shows up at once (the origin, and what the page
    // stores, stay the same).
    let url: Url = format!("{}/?v={}", server.url, env!("CARGO_PKG_VERSION")).parse()?;

    let home = url.origin();
    WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
        .title("Russian Cases Trainer")
        .inner_size(1200.0, 820.0)
        .min_inner_size(720.0, 560.0)
        .on_navigation(move |to| {
            if to.origin() == home {
                return true;
            }
            // OpenRussian, Wiktionary, Piper…: the user's browser.
            let _ = tauri_plugin_opener::open_url(to.as_str(), None::<&str>);
            false
        })
        .build()
        .context("opening the window")?;

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        match app.updater() {
            Ok(updater) => match updater.check().await {
                Ok(Some(update)) => offer(app, update),
                Ok(None) => {}
                Err(e) => tracing::info!(error = %e, "no update information"),
            },
            Err(e) => tracing::warn!(error = %e, "updater unavailable"),
        }
    });
    Ok(())
}

/// The port used last time, else the preferred one, else any free one. Kept
/// in `<data>/port`.
fn free_address(data_dir: &Path) -> SocketAddr {
    let file = data_dir.join("port");
    let last = std::fs::read_to_string(&file)
        .ok()
        .and_then(|port| port.trim().parse().ok());
    let port = last
        .into_iter()
        .chain([PREFERRED_PORT, 0])
        .find_map(|port| {
            TcpListener::bind((Ipv4Addr::LOCALHOST, port))
                .and_then(|listener| listener.local_addr())
                .ok()
                .map(|addr| addr.port())
        })
        .unwrap_or(0);
    let _ = std::fs::create_dir_all(data_dir);
    let _ = std::fs::write(&file, port.to_string());
    (Ipv4Addr::LOCALHOST, port).into()
}

fn offer(app: AppHandle, update: Update) {
    let t = texts();
    app.dialog()
        .message((t.update)(&update.version))
        .title("Russian Cases Trainer")
        .buttons(MessageDialogButtons::OkCancelCustom(
            t.install.into(),
            t.later.into(),
        ))
        .show(move |install| {
            if !install {
                return;
            }
            tauri::async_runtime::spawn(async move {
                match update.download_and_install(|_, _| {}, || {}).await {
                    Ok(()) => app.restart(),
                    Err(e) => tracing::warn!(error = %e, "the update failed"),
                }
            });
        });
}

/// Logs go to `desktop.log` in the app's log folder.
fn log_to(dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("desktop.log"))?;
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .with_ansi(false)
        .with_writer(std::sync::Mutex::new(file))
        .try_init()
        .map_err(|e| anyhow::anyhow!(e))
}

/// The few texts shown before the page is there, in the system's language.
struct Texts {
    failed: &'static str,
    update: fn(&str) -> String,
    install: &'static str,
    later: &'static str,
}

fn texts() -> Texts {
    let locale = sys_locale::get_locale().unwrap_or_default();
    match locale.get(..2).unwrap_or("en") {
        "fr" => Texts {
            failed: "L’application n’a pas pu démarrer.",
            update: |v| format!("La version {v} est disponible. L’installer maintenant ?"),
            install: "Installer",
            later: "Plus tard",
        },
        "de" => Texts {
            failed: "Die App konnte nicht starten.",
            update: |v| format!("Version {v} ist verfügbar. Jetzt installieren?"),
            install: "Installieren",
            later: "Später",
        },
        "ru" => Texts {
            failed: "Не удалось запустить приложение.",
            update: |v| format!("Доступна версия {v}. Установить сейчас?"),
            install: "Установить",
            later: "Позже",
        },
        _ => Texts {
            failed: "The app couldn't start.",
            update: |v| format!("Version {v} is available. Install it now?"),
            install: "Install",
            later: "Later",
        },
    }
}
