# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# Use latest stable as default.
# When overriding, use the alpine version to ensure
# no dependency on glibc, libgcc, etc.
# Although my license is GPL-compatible, which means
# I could technically statically link against GPL-3
# code, fuck that shit. It uses dlopen to load *itself*
# at runtime so you essentially end up with two libc's
# in memory, which.. yeah no.
# I refuse to bow down to the morons at FSF.
ARG RUST_VERSION=alpine3.19
FROM rust:${RUST_VERSION} AS build

# Install a few essential packages
#  - build-base: to build jemalloc
#  - musl-dev: C standard library
#  - git: required by build.rs
RUN apk add --no-cache build-base=~0 musl-dev=~1 git=~2

# shell options
#   -e: exit on error
#   -u: error on unset variables
#   -x: print each command
SHELL [ "/bin/sh", "-eux", "-c" ]

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

# Feature flags to enable
ARG FEATURES=default
ENV SQLX_OFFLINE=1 FEATURES=${FEATURES}
## Build dependencies separately to cache them
RUN mkdir src && \
  echo 'fn main(){panic!("If you see this, the build broke.")}' > src/main.rs && \
  cargo build --release --features ${FEATURES}
## Build the actual binary
COPY . .
RUN cargo build --release --features ${FEATURES}

####

FROM scratch

# Labels
# Reference: https://github.com/opencontainers/image-spec/blob/main/annotations.md
LABEL org.opencontainers.image.source "https://github.com/lmaotrigine/heartbeat"
LABEL org.opencontainers.image.authors "root@5ht2.me"
LABEL org.opencontainers.image.title "heartbeat"
LABEL org.opencontainers.image.description "A service to show a live digital heartbeat (ping) on multiple devices."
LABEL org.opencontainers.image.licenses "MPL-2.0"

COPY --from=build /usr/src/app/target/release/heartbeat /.heartbeat/bin/heartbeat
ENV HEARTBEAT_HOME=/.heartbeat

# test if the binary works
RUN [ "/.heartbeat/bin/heartbeat", "--version" ]
ENTRYPOINT [ "/.heartbeat/bin/heartbeat" ]
