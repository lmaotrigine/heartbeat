ARG RUST_VERSION=latest

FROM rust:${RUST_VERSION} AS base
RUN cargo install cargo-chef
WORKDIR /usr/src/app

FROM base AS inter
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS build
ARG FEATURES=default
COPY --from=inter /usr/src/app/recipe.json recipe.json
ENV PKG_CONFIG_ALLOW_CROSS=1 SQLX_OFFLINE=1 FEATURES=${FEATURES}
RUN cargo chef cook --release --features ${FEATURES} --bin heartbeat --recipe-path recipe.json
COPY . .
RUN cargo build --release --features ${FEATURES} --bin heartbeat

####

FROM gcr.io/distroless/cc-debian12

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
