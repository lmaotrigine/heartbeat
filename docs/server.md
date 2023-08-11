# Server Setup

## Configuration

Configuration for the server is read from a `config.toml` file in the working directory. An example configuration is
[distributed with the source](/config.example.toml).

All fields in the file have comments explaining what they are for, and how to fill them out.

## Running

### Docker

Docker images are built and pushed on every commit to the `main` branch. A [`docker-compose.yml`](/docker-compose.yml)
file is included in the source to quickly get a server set up and running.

```sh
docker compose up
```

If you know what you're doing, you can edit the compose file to use, say, a database on a different host/network.

### Pre-built binaries

Pre-built, statically linked binaries are available as compressed archives on the
[releases page](https://github.com/lmaotrigine/heartbeat/releases). Pick the suitable
one for your system and extract it.

### Build from source

See the [optional features](./usage.md#optional-features) section for feature flags.

```sh
cargo build --release # add --feature flags here
ln -s target/release/heartbeat ./heartbeat
./heartbeat
```

In addition, you will need access to a [PostgreSQL](https://www.postgresql.org) (of a supported server version).
The schemas for the database are located in the [`migrations`](/migrations) directory.


## Running in production

It is recommended to use a robust webserver like Nginx/Caddy to reverse proxy connections to heartbeat. Caddy supports
HTTPS by default, whereas Nginx requires additional configuration to include SSL certificates and keys. See their docs
for details. Sample configuration files are included in the [`conf`](/conf) directory.
