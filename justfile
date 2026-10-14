set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

rust_image := "rust:1-slim-trixie"

default:
    @just --list

# Build and start db, migrate, backend and frontend
up:
    docker compose up -d --build

down:
    docker compose down

logs *args:
    docker compose logs -f {{args}}

# Apply database migrations
migrate:
    docker compose run --rm migrate

# Download the OpenRussian word lists and load nouns + rules
import:
    docker compose --profile tools run --rm --build importer

# DB web UI on http://localhost:8081
adminer:
    docker compose --profile tools up -d adminer

# Run any cargo command in a Rust container, e.g. `just cargo clippy --workspace`
cargo *args:
    docker run --rm --network russian-cases-trainer_default -e DATABASE_URL=postgres://trainer:trainer@db:5432/trainer -v "{{justfile_directory()}}/backend:/app" -v rct-cargo-registry:/usr/local/cargo/registry -v rct-target:/app/target -w /app {{rust_image}} cargo {{args}}

test:
    just cargo test --workspace

# Regenerate the typed API client from the running backend's OpenAPI spec
gen-api:
    cd frontend; npx --yes -p openapi-typescript@7 -p typescript@5 openapi-typescript http://localhost:8080/api/openapi.json -o src/shared/api/schema.d.ts
