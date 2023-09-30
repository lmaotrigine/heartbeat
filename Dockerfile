# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# Use latest stable as default.
ARG RUST_VERSION=bullseye

FROM rust:${RUST_VERSION} AS build

# shell options
#   -e: exit on error
#   -u: error on unset variables
#   -x: print each command
#   -o pipefail: fail if any command in a pipe fails
SHELL [ "/bin/bash", "-euxo", "pipefail", "-c" ]

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

# Feature flags to enable
ARG FEATURES=default
ENV SQLX_OFFLINE=1 FEATURES=${FEATURES}
## Build dependencies separately to cache them
RUN mkdir -p src/bin; \
  echo 'fn main(){println!("If you see this, the build broke.")}' \
    | tee src/bin/web.rs src/bin/migrate_db.rs > src/bin/generate_secret.rs; \
  cargo build --release --features ${FEATURES} --bin heartbeat
## Build the actual binary
COPY . .
RUN cargo build --release --features ${FEATURES} --bin heartbeat

####

FROM gcr.io/distroless/cc-debian11:nonroot

# Log level for tracing-subscriber
ARG RUST_LOG=info

# Labels
# Reference: https://github.com/opencontainers/image-spec/blob/main/annotations.md
LABEL org.opencontainers.image.source "https://github.com/lmaotrigine/heartbeat"
LABEL org.opencontainers.image.authors "root@5ht2.me"
LABEL org.opencontainers.image.title "heartbeat"
LABEL org.opencontainers.image.description "A service to show a live digital heartbeat (ping) on multiple devices."
LABEL org.opencontainers.image.licenses "MPL-2.0"

COPY --from=build /usr/src/app/target/release/heartbeat /usr/local/bin/heartbeat
COPY --from=build /usr/src/app/static /usr/local/share/heartbeat/static

WORKDIR /usr/local/share/heartbeat
ENV RUST_LOG=${RUST_LOG}
CMD [ "/usr/local/bin/heartbeat" ]
