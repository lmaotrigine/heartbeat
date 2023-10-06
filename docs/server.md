# Server Setup

## Configuration

Configuration for the server is hierarchical and read from the following sources.

- A TOML configuration file located at `config.toml` in the current working directory, or the location specified by the
  environment varable `HEARTBEAT_CONFIG_FILE` or the command line option `-c`/`--config-file`.
- Environment variables prefixed with `HEARTBEAT_`.
- Command line options.

In case of conflicting options, command line options take precedence over environment variables, which in turn take
precedence over TOML configuration fields.

An example configuration file is [distributed with the source](/config.example.toml).

All fields in the file have comments explaining what they are for, and how to fill them out.

Help for the options is also available the obvious way:

```console
$ cargo run -q --bin heartbeat -- --help # cut below: version and author information
Usage: heartbeat [OPTIONS]

Options:
  -d, --database-dsn <DATABASE_DSN>    A PostgreSQL connection string [env: HEARTBEAT_DATABASE_URL=]
      --webhook-url <WEBHOOK_URL>      The URL of the Discord webhook [env: HEARTBEAT_WEBHOOK_URL=]
      --webhook-level <WEBHOOK_LEVEL>  The minimum level of events that triggers a webhook [env: HEARTBEAT_WEBHOOK_LEVEL=]
  -s, --secret-key <SECRET_KEY>        A random URL-safe string used as a master Authorization header for adding new devices [env: HEARTBEAT_SECRET_KEY=]
  -r, --repo <REPO>                    The GitHub repository URL of the project [env: HEARTBEAT_REPO=]
      --server-name <SERVER_NAME>      A human-readable name for the server used in <title> tags and other metadata [env: HEARTBEAT_SERVER_NAME=]
  -u, --live-url <LIVE_URL>            The publicly accessible URL of the server [env: HEARTBEAT_LIVE_URL=]
  -b, --bind <BIND>                    The bind address for the server. Must be parsable by [`std::net::ToSocketAddrs`] [env: HEARTBEAT_BIND=]
      --static-dir <STATIC_DIR>        Path to the directory containing static files. [default: ./static] [env: HEARTBEAT_STATIC_DIR=]
  -c, --config-file <CONFIG_FILE>      The path to the configuration file. [default: ./config.toml] [env: HEARTBEAT_CONFIG_FILE=]
  -h, --help                           Print help
  -V, --version                        Print version
```

## Running

### Docker

Docker images are built and pushed on every commit to the `main` branch. A [`docker-compose.yml`](/docker-compose.yml)
file is included in the source to quickly get a server set up and running.

```console
$ docker compose up
```

If you know what you're doing, you can edit the compose file to use, say, a database on a different host/network.

### Build from source

You will require a Rust toolchain to build from source. The Minimum Supported Rust Version is 1.70. However, this is
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
