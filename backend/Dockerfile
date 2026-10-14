# Multi-stage build: cargo-chef caches the dependency layer so code edits rebuild fast.

# toolbox: full toolchain for `just cargo ...` (tests, clippy, fmt) with the source mounted.
FROM rust:1-slim-trixie AS toolbox
RUN rustup component add clippy rustfmt
WORKDIR /app

FROM rust:1-slim-trixie AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin server --bin importer

FROM debian:trixie-slim AS runtime
# /data is the importer's download cache; a fresh named volume inherits its ownership.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home app \
    && mkdir /data && chown app:app /data
COPY --from=builder /app/target/release/server /app/target/release/importer /usr/local/bin/
USER app
WORKDIR /home/app
EXPOSE 8080
CMD ["server", "serve"]
