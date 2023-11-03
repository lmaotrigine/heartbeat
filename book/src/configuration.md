# Configuration

This document explains how Heartbeat's configuration system works, as well as available keys or configuration.

## Hierarchical structure

Heartbeat allows configuration to be specified through a file, through environment variables, or in the command line
during invocation. If a key is specified in multiple locations, the value in the location with highest priority will
take precedence. The locations in decreasing order of priority are:

- Command line
- Environment variables
- Config file fields

## Specifying the configuration file

By default, Heartbeat looks for a configuration file called `config.toml` in the directory it was invoked from. If the
file doesn't exist, it silently moves on. This can be overridden by the environment variable `HEARTBEAT_CONFIG_FILE`, or
the command line flag `-c`/`--config-file`. If a regular file is not found at the location pointed to by the value of
this parameter, an error will be printed and Heartbeat will exit.

## Configuration format

Configuration files are written in the [TOML format][toml], with simple key-value pairs and sections (tables). Values
can be specified based on [profiles](https://doc.rust-lang.org/stable/cargo/reference/profiles.html) using the `debug`
or `release` tables, both of which are optional. An example config file with all fields filled out is given below:

```toml
# the address to bind to
# this will be parsed as a `std::net::SocketAddr`
bind = "0.0.0.0:6060"

# this is used for <title> tags
# and some headings
server_name = "Some person's heartbeat"

# the full url to the server
live_url = "http://127.0.0.1:6060"

# a random URL-safe string.
# if left blank, addition of devices is disabled.
# this may be generated using `openssl rand -base64 45`
secret_key = ""

# don't change this unless you're using a fork
repo = "https://github.com/lmaotrigine/heartbeat"

[database]
# this is the default if you're using the docker-compose file
# otherwise, the format is postgresql://username:password@host/database
dsn = "postgresql://heartbeat@db/heartbeat"

[webhook]
# leave this blank to disable webhooks
url = ""

# one of:
# - "all"             log each beat + the below
# - "new_devices"     log new devices + the below
# - "long_absences"   log absences longer than 1 hour
# - "none"            don't log anything
level = "none"

# override some values for debug builds for easier testing.

[debug]
bind = "127.0.0.1:6061"
live_url = "http://localhost:6061"

# nested keys work as expected
[debug.database]
dsn = "postgresql://heartbeat@localhost/heartbeat"
```

## Environment variables

Heartbeat can be configured through environment variables in addition to the TOML configuration file. Environment
variable names take the form of `HEARTBEAT_<KEY>` where `<KEY>` is the full path to the key in the TOML configuration,
in uppercase, with dots (`.`) replaced with underscores (`_`). For example, the `database.dsn` config value can be
specified through the environment variable `HEARTBEAT_DATABASE_DSN`. Environment variables will override the values in
the TOML configuration file, regardless of profile. Environment variable names must not contain the profile key.

## Command line arguments

Heartbeat can be configured by passing configuration values as command line parameters. Command line flags are formatted
as per the POSIX standard, with two leading dashes (`--`) and dots (`.`) and underscores (`_`) replaced with dashes
(`-`). For example, the `database.dsn` config value can be passed via the command line argument `--database-dsn=`. The
equals sign is optional. For a full list of available configuration values, their usage, default values, and available
shorthands, run `heartbeat --help`.

If any required configuration parameter is not specified (all parameters with no defaults are considered required),
Heartbeat will print an error message and exit.

## Configuration keys

This section documents all configuration keys.

### `[database]`

the `[database]` table contains configuration related to the PostgreSQL database

#### `database.dsn`

- Type: string
- Default: `postgresql://heartbeat@db/heartbeat` if running within Docker, `postgresql://postgres@localhost/postgres`
  otherwise.
- Enviroment: `HEARTBEAT_DATABASE_DSN`
- Command line: `-d`/`--database-dsn`

The PostgreSQL connection string for the database that Heartbeat should use. The database must exist and the user must
have at least `CREATE SCHEMA` privileges on it.

### `[webhook]`

The `[webhook]` table deals with configuration related to logging events to Discord webhooks. This is only relevant if
the `webhooks` feature is enabled, which is the default.

#### `webhook.url`

- Type: string
- Default: empty
- Environment: `HEARTBEAT_WEBHOOK_URL`
- Command line: `--webhook-url`

The URL to the Discord webhook to log events to. If empty, logging is disabled.

#### `webhook.level`

- Type: string, one of `all`, `new_devices`, `long_absences`, `none`
- Default: `none`
- Environment: `HEARTBEAT_WEBHOOK_LEVEL`
- Command line: `--webhook-level`

The maximum level of events to log to the webhook. The possible values in descending order of level are:

- `all`: logs beats, along with all of the below
- `new_devices`: logs when a new device is added, along with all of the below
- `long_absences`: logs when an absence longer than 1 hour has ended.
- `none`: No events are logs.

### `secret_key`

- Type: string
- Default: none
- Environment: `HEARTBEAT_SECRET_KEY`
- Command line: `-s`/`--secret-key`

A random, header value safe string (<=256 bytes) that will be the master authentication token for administrative actions
like adding devices or regenerating their tokens. If this value is empty, administrative actions are disabled.

### `repo`

- Type: string
- Default: `https://github.com/lmaotrigine/heartbeat`
- Environment: `HEARTBEAT_REPO`
- Command line: `-r`/`--repo`

The URL to the repository of the Heartbeat source. This should be left unchanged unless you are running a fork.

### `server_name`

- Type: string
- Default: `Some person's heartbeat`
- Environment: `HEARTBEAT_SERVER_NAME`
- Command line: `--server-name`

A human-readable name for the server. Used in `<title>` tags and other metadata.

### `live_url`

- Type: string
- Default `http://127.0.0.1:6060`
- Environment: `HEARTBEAT_LIVE_URL`
- Command line: `-u`/`--live-url`

The publicly accessibly base URL for the Heartbeat server. This is used to format URLs in webhook logs so that they
resolve to the right routes. When running in production, this must be overridden.

### `bind`

- Type: string, must be a valid socket address (either `<host>:<port>`, or a UDS).
- Default: `127.0.0.1:6060`
- Environment: `HEARTBEAT_BIND`
- Command line: `-b`/`--bind`

The socket address for the server to bind to and listen on.

### `config_file`

- Type: string, must be a path to a valid file
- Default: `./config.toml`
- Environment: `HEARTBEAT_CONFIG_FILE`
- Command line: `-c`/`--config-file`

This value obviously cannot be specified in the TOML configuration file. It can be set in the environment or provided in
the command line to override the default config file location.


[toml]: https://toml.io
