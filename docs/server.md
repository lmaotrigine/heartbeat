# Server Setup

## Configuration

Configuration for the server is read from a `config.toml` file in the working directory. An example configuration is
[distributed with the source](/config.example.toml).

All fields in the file have comments explaining what they are for, and how to fill them out.

## Running

### Docker

Docker images are built and pushed on every commit to the `main` branch. A [`docker-compose.yml`](/docker-compose.yml)
file is included in the source to quickly get a server set up and running.

```console
$ docker compose up
```

If you know what you're doing, you can edit the compose file to use, say, a database on a different host/network.

### Build from source

You will require a Rust toolchain to build from source. The Minimum Supported Rust Version is 1.69. However, this is
subject to change at any time, so make sure you have the latest stable toolchain installed anyway. This crate is not
intended for use as a library, so there is no guarantee for the MSRV.

See the [feature flags](./usage.md#feature-flags) section for feature flags.

```console
$ cargo build --release # add --feature flags here
$ ln -s target/release/heartbeat ./heartbeat
$ ./heartbeat
```

In addition, you will need access to a [PostgreSQL](https://www.postgresql.org) database server (of a supported
version). If the server requires SSL/TLS connections, you will need to build with the `sqlx-tls` feature flag to enable
support for this. Migrations are embedded in the `migrate_db` binary, and running this will apply all pending
migrations. For more details on managing migrations, refer to the [SQLX CLI
docs](https://github.com/launchbadge/sqlx/blob/v0.7.1/sqlx-cli/README.md).

For starting fresh, you can simple use the [`init.sql`](/docker-entrypoint-initdb.d/init.sql) script to create the
necessary tables and indexes in your database.


## Running in production

It is recommended to use a robust webserver like Nginx/Caddy to reverse proxy connections to heartbeat. Caddy supports
HTTPS by default, whereas Nginx requires additional configuration to include SSL certificates and keys. See their docs
for details. Sample configuration files are included in the [`conf`](/conf) directory.
