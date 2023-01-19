FROM rust:1.66.1 AS build

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV SQLX_OFFLINE=1

WORKDIR /usr/src/app
COPY . .
RUN cargo install --all-features --path .

####

FROM gcr.io/distroless/cc-debian11

COPY --from=build /usr/local/cargo/bin/heartbeat /usr/local/bin/heartbeat
COPY --from=build /usr/src/app/Rocket.toml /usr/local/share/heartbeat/Rocket.toml
COPY --from=build /usr/src/app/static /usr/local/share/heartbeat/static
COPY --from=build /usr/src/app/templates /usr/local/share/heartbeat/templates

WORKDIR /usr/local/share/heartbeat
CMD [ "/usr/local/bin/heartbeat" ]
