# Server Setup

## Configuration

Configuration for the server is read from a `config.toml` file in the working directory. An example configuration is
[distributed with the source](/config.example.toml).

All fields in the file have comments explaining what they are for, and how to fill them out.

Configuration specific to the webserver (such as listen address and port) are specified instead in
[`Rocket.toml`](/Rocket.toml). See the [documentation](https://rocket.rs/v0.5-rc/guide/configuration/)
for more information.

## Running

### Docker

Docker images are built and pushed on every commit to the `main` branch. A [`docker-compose.yml`](/docker-compose.yml)
file is included in the source to quickly get a server set up and running.

```sh
docker compose up
```

If you know what you're doing, you can edit the compose file to use, say, a database on a different host/network.

### Binary

> :warning: This section deals with release binaries. I am not sure if I will be tagging releases anytime soon, so
> if you don't want to use Docker, skip ahead to [building from source](#build-from-source).

Binaries are built on every tag pushed. You can grab the one suited to your platform and architecture from the
[releases](https://github.com/lmaotrigine/heartbeat/releases) page. If a binary is not available for the commit you
need, you will have to [build from source](#build-from-source).

In addition, you will need access to a [PostgreSQL](https://www.postgresql.org) (of a supported server version).
The schema for the database is located in the [`migrations`](/migrations/20230103063306_initial_migration.sql) directory.

If you want to change the default address/port, edit the corresponding settings in the `[release]` section of
[`Rocket.toml`](/Rocket.toml).

### Build from source

Minimum Supported Rust Version: 1.66.1

See the [optional features](./usage.md#optional-features) section for feature flags.

```sh
cargo build --release # add --feature flags here
ln -s target/release/heartbeat ./heartbeat
./heartbeat
```

## Running in production

It is recommended to use a robust webserver like Nginx/Caddy to reverse proxy connections to heartbeat. Caddy supports
HTTPS by default, whereas Nginx requires additional configuration to include SSL certificates and keys. See their docs
for details. Sample configuration files are included in the [`conf`](/conf) directory.
