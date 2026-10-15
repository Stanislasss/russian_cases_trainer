set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

default:
    @just --list

# Build and start db, migrate, backend, frontend and tts
up:
    docker compose up -d --build

down:
    docker compose down

logs *args:
    docker compose logs -f {{args}}

# Apply database migrations
migrate:
    docker compose run --rm migrate

# Download the word lists and translations, then load them
import:
    docker compose --profile tools run --rm --build importer

# Write the desktop app's word list to build/lexicon.sqlite (same downloads as `import`)
lexicon-sqlite:
    docker compose --profile tools run --rm --build --no-deps -v ./build:/out importer importer --sqlite /out/lexicon.sqlite

# The desktop app's backend in Docker, on http://localhost:8090 (needs `just lexicon-sqlite`)
serve-local:
    cd frontend; npm run build
    docker compose --profile tools up -d --build local

# DB web UI on http://localhost:8081
adminer:
    docker compose --profile tools up -d adminer

# Run any cargo command in a Rust container, e.g. `just cargo clippy --workspace`
cargo *args:
    docker compose --profile tools run --rm cargo {{args}}

test:
    just cargo test --workspace --exclude desktop

# fmt check + clippy + tests, as CI runs them
check:
    just cargo fmt --all --check
    just cargo clippy --workspace --exclude desktop --all-targets -- -D warnings
    just test

# Regenerate the typed API client from the running backend's OpenAPI spec
gen-api:
    cd frontend; npx --yes -p openapi-typescript@7 -p typescript@5 openapi-typescript http://localhost:8080/api/openapi.json -o src/shared/api/schema.d.ts
