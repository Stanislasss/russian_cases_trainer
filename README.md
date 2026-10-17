# Russian Cases Trainer

An app for practising the six Russian grammatical cases: browse the declension rules case by case, then drill them by dragging each word onto the right ending. It runs as a web app or as a [desktop app](#desktop-app) for Windows, macOS and Linux.

- **Backend:** Rust (axum + sqlx), hexagonal workspace in [`backend/`](backend)
- **Frontend:** React + TypeScript + Vite in [`frontend/`](frontend)
- **Database:** PostgreSQL 17 (web), SQLite (desktop)
- **UI languages:** English, French, German, Russian
- **Accounts (web):** email and password; guests can play, but results are saved only when logged in. The desktop app has one local player whose results are always saved.

## Getting started

Requires Docker Desktop and [`just`](https://github.com/casey/just).

```sh
cp .env.example .env
just up        # db, migrations, backend (:8080), frontend (:5173), tts (:5002)
just import    # load the words and their translations
```

Open http://localhost:5173.

## Endings game

The game (`/play/endings`) shows a word in its dictionary form and the case to put it in; drag it onto the right ending.

- **Levels** (checkboxes): basic, intermediate and advanced nouns; basic and intermediate adjectives; possessive and demonstrative pronouns; surnames. Personal pronouns are played on their own, with whole forms (мной, ему…) instead of endings.
- **Practice:** one word at a time, read aloud. A wrong answer pauses the game and shows the rule that applies, with the word kept on screen, until you click Continue. After a right answer the correct form is spoken, then the next word comes after a short pause (0–5 s, chosen on the setup screen; Enter or Next skips it).
- **Arcade:** words fall against the clock; mistakes are explained in the summary.
- **Translations** follow the UI language: English, German, French, or a Russian definition. English is the fallback when a word has none.

## Sentences game

The sentence game (`/play/sentences`) shows a short sentence with one noun in its dictionary form. First pick the **case** the sentence needs, then the **ending**. A wrong case is explained with the use that requires it (e.g. "After к, по, благодаря́ …") and what the chosen case is used for instead; a wrong ending shows the same rule card as the endings game. A right answer is read aloud.

The sentences are frames in `backend/crates/domain/src/sentences.rs`, about 40 of them, each with its case, the reason (a use id from the frontend's `cases.ts`), allowed numbers, animacy and translations. They're filled with random nouns from the chosen levels, so frames are chosen to work with almost any noun ("Я ду́маю о ___", "У меня́ нет ___"; "Я позвони́л ___" only takes people and animals). A few sentences still come out quirky ("Я бою́сь но́са"). Place frames (в ___) are left out for that reason.

## Pronunciation

Pronunciations come from the `tts` service, a local [Piper](https://github.com/OHF-Voice/piper1-gpl) server (GPL-3.0) with four Russian voices. Compare them on http://localhost:5002 and set `TTS_VOICE` in `.env` (then `just up`). Audio is cached in the `speech-cache` volume; without the service the game simply has no sound. The 🔊 buttons and the automatic reading in Practice can be muted.

## Desktop app

A [Tauri](https://tauri.app) app (`backend/crates/desktop`) that runs the same backend inside itself on `127.0.0.1` and shows the frontend in the system's webview. It works offline and needs no Postgres:

- **Words:** `lexicon.sqlite`, a read-only SQLite word list shipped with the app (about 25 MB, only the words some level can ask), written by `importer --sqlite`.
- **Progress:** `progress.sqlite` in the user's app-data folder, with one local player; there are no accounts.
- **Pronunciation is optional:** Settings → Pronunciation downloads the *speech pack* (about 70 MB): native Piper, the Russian espeak-ng data of piper-tts (Piper's own ignores stress marks) and the voice ru_RU-irina-medium. Until then there are no 🔊 buttons. The pack's download addresses and checksums are in `backend/speech-pack.json`.
- **Updates:** at start, the app looks for a newer release and offers to install it.

Installers come from the public [releases repository](https://github.com/Stanislasss/russian-cases-trainer-releases). They aren't signed yet, so the first launch shows a warning: on Windows click *More info → Run anyway*; on macOS right-click the app → *Open*.

### Building and releasing

Your machine doesn't need a Rust toolchain: everything but the window runs in Docker, and the app itself is built on GitHub Actions.

- `just lexicon-sqlite` writes `build/lexicon.sqlite` (same downloads as `just import`).
- `just serve-local` builds the frontend and runs the desktop backend in Docker on http://localhost:8090: SQLite, the local player, the settings page.
- **CI** (`.github/workflows/ci.yml`) runs the backend and frontend checks and compiles the desktop app on Linux.
- **Release** (`release.yml`): set `version` in `backend/Cargo.toml`, commit, push the tag `v<version>`. It builds the word list, then the installers for Windows, macOS (Apple silicon and Intel) and Linux, and creates a **draft** release in the releases repository. Publishing the draft makes installed apps update to it.
- **Speech pack** (`speech-pack.yml`, run by hand): packs Piper per system, checks on each one that stress marks are honoured, publishes the parts, and prints the `speech-pack.json` to commit.

One-time setup: create the public repository `russian-cases-trainer-releases`, generate the updater key with `npx @tauri-apps/cli signer generate` (put its public key in `backend/crates/desktop/tauri.conf.json`, keep the private key and its password backed up offline: without them installed apps can't be updated), and add the repository secrets `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` and `RELEASES_REPO_TOKEN` (a fine-grained token with *Contents: read and write* on the releases repository only).

## Commands

| Command | What it does |
|---|---|
| `just up` / `just down` | Start / stop the stack |
| `just logs [service]` | Follow logs |
| `just migrate` | Apply database migrations |
| `just adminer` | Database web UI on http://localhost:8081 |
| `just test` | Run the backend tests in a Rust container |
| `just check` | Backend fmt check + clippy + tests |
| `just cargo <args>` | Any cargo command in a Rust container |
| `just gen-api` | Regenerate the TypeScript API types from the running backend |
| `just lexicon-sqlite` | Write the desktop app's word list to `build/lexicon.sqlite` |
| `just serve-local` | Run the desktop app's backend in Docker on http://localhost:8090 |

Frontend checks run natively from `frontend/`: `npm run typecheck`, `npm run lint`, `npm test`.

## Grammar rules

The **Rules** section (`/rules`) explains what each case is used for, its endings and its exceptions, in English, French, German and Russian. The content lives in `frontend/src/shared/grammar/`:

- `paradigms.ts` — ending tables per declension pattern (language-independent);
- `cases.ts` — uses and exceptions per case, with Russian examples;
- `texts/{en,fr,de,ru}.ts` — the explanations, typed so a missing translation fails the type check.

The backend tags every game word with its pattern (`domain::rules::paradigm`), so a wrong answer can show exactly the rule that applies.

## Backend layout

| Crate | Role |
|---|---|
| `domain` | Grammar types and pure logic, no I/O |
| `application` | Use cases and the ports (traits) they depend on |
| `infrastructure` | Postgres and SQLite repositories, the speech pack, password hashing, migrations |
| `api` | axum routes, DTOs, OpenAPI document |
| `server` | Binary: config and wiring (`serve`, `serve-local`, `migrate`, `print-openapi`); library: the desktop backend |
| `importer` | Binary: loads the OpenRussian word lists, the Wiktionary translations and the hand-written words (into Postgres, or `--sqlite`) |
| `desktop` | The Tauri app; built in CI only |

The Postgres and SQLite repositories share their tests (`infrastructure/src/contract.rs`), so both behave the same.

## Data attribution

- Nouns, adjectives, pronouns and their English and German translations come from [OpenRussian.org](https://en.openrussian.org), licensed under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
- French translations and Russian definitions come from the [French](https://fr.wiktionary.org) and [Russian](https://ru.wiktionary.org) Wiktionary via [kaikki.org](https://kaikki.org), licensed under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). `just import` downloads them (176 MB and 1.7 GB, cached); if a download fails, that language falls back to English.
- Personal pronouns, наш/ваш and the surname list are written by hand in `backend/crates/importer/src/curated.rs`; surnames are declined by rule (`domain::surname`).

## License

Apache-2.0. The desktop app's speech pack, downloaded separately, contains Piper and eSpeak NG (GPL-3.0).
