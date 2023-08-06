FROM rust:1.71.0 AS base
RUN cargo install cargo-chef
WORKDIR /usr/src/app

FROM base AS inter
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS build
COPY --from=inter /usr/src/app/recipe.json recipe.json

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV SQLX_OFFLINE=1

RUN cargo chef cook --release --features badges,webhook --recipe-path recipe.json
COPY . .
RUN cargo build --release --features badges,webhook

####

FROM gcr.io/distroless/cc-debian11

COPY --from=build /usr/src/app/target/release/heartbeat /usr/local/bin/heartbeat
COPY --from=build /usr/src/app/static /usr/local/share/heartbeat/static

WORKDIR /usr/local/share/heartbeat
ENV RUST_LOG=info
CMD [ "/usr/local/bin/heartbeat" ]
