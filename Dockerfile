ARG RUST_VERSION=bullseye

FROM rust:${RUST_VERSION} AS build

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

ARG FEATURES=default
ENV PKG_CONFIG_ALLOW_CROSS=1 SQLX_OFFLINE=1 FEATURES=${FEATURES}
## Explicitly build dependencies to cache them
RUN \
  set -eux; \
  mkdir -p src/bin; \
  echo 'fn main() {println!("If you see this, the build broke.")}' \
    | tee src/bin/web.rs src/bin/migrate_db.rs > src/bin/generate_secret.rs; \
  cargo build --release --features ${FEATURES} --bin heartbeat
## Build the actual binary
COPY . .
RUN cargo build --release --features ${FEATURES} --bin heartbeat

####

FROM gcr.io/distroless/cc-debian11

ARG RUST_LOG=info

LABEL org.opencontainers.image.source "https://github.com/lmaotrigine/heartbeat"
LABEL org.opencontainers.image.authors "root@5ht2.me"
LABEL org.opencontainers.image.title "heartbeat"
LABEL org.opencontainers.image.licenses "MPL-2.0"
LABEL org.opencontainers.image.base.name "gcr.io/distroless/cc-debian11"

COPY --from=build /usr/src/app/target/release/heartbeat /usr/local/bin/heartbeat
COPY --from=build /usr/src/app/static /usr/local/share/heartbeat/static

WORKDIR /usr/local/share/heartbeat
ENV RUST_LOG=${RUST_LOG}
CMD [ "/usr/local/bin/heartbeat" ]
