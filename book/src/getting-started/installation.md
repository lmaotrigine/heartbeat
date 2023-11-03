# Installation

## Installing Heartbeat

The easiest way to install heartbeat is by obtaining the latest release archive from the [releases]. These release
binaries are statically linked and require no runtime dependencies. You may simply download the archive corresponding to
your platform and extract it.

### Docker

First-party tagged and annotated Docker images are built on every commit to the `main` branch and on every tag and can
be pulled from the GitHub Container Registry.

```console
$ docker pull ghcr.io/lmaotrigine/heartbeat:latest
```

Valid tags are:

- `latest`: Always points to the tip of `main`. Breaking changes may occur unannounced.
- The first 7 characters of a commit on `main` (corresponding to the output of `git rev-parse --short`).
- Any tag corresponding to a tagged release (e.g. `0.1.0`)

#### Customizing the Image

You may optionally pass these additional [build arguments] while
building the Heartbeat Docker image yourself:

- `RUST_VERSION`: defaults to `alpine`, which is the latest stable. Please make sure to use an Alpine base since it
  allows the C runtime to be linked statically. `glibc`-based distros like Debian will make the final image a lot
  bigger. However, Heartbeat's [license] is GPL-compatible, should you wish to statically link against `glibc`.
- `FEATURES`: the features to enable in the resultant binary. By default, only the default feature set is enabled. The
  only relevant setting for overriding this is `sqlx-tls`, which allows you to connect to your PostgreSQL database over
  TLS.

### Build and Install Heartbeat from Source

Alternatively, you can build heartbeat from source.

#### Requirements

Heartbeat requires the following tools and packages to build:

- `cargo` and `rustc`
- A C compiler [for your platform](https://github.com/rust-lang/cc-rs#compile-time-requirements)
- `git` (to clone this repository and also to attach some build metadata to the resulting library)

#### Compiling

First, you'll want to check out this repository:

```console
$ git clone https://github.com/lmaotrigine/heartbeat.git
$ cd heartbeat
```

With `cargo` installed, you can simply run:

```console
$ cargo build --release
```

#### Features

Some functionality is gated behind [feature flags]. All features are enabled in the pre-built binaries.

- `badges`: Enables support for the `/badge/*` routes. This enables generation of SVG badges in the style of
  [shields.io], without having to write long URLs for the dynamic badges that shields.io provides. Enabled by default.
- `webhook`: Enables logging selected events to a Discord webhook. Enabled by default.
- `migrate`: Required to run the embedded database migrations. You will need to run this if the database schema is
  changed at some point. Such changes will be considered breaking and backwards incompatible. The migrations will help
  you to upgrade from previous versions of the schema.
- `sqlx-tls`: Allows you to connect to a PostgreSQL server over TLS. Such a setup is not necessary if your database
  instance is running on a local network and is not exposed publicly.

## Setting up the Database

Heartbeat uses a [PostgreSQL] database to store statistics, device information and beat history. This was chosen due to
it being battle-tested and robust and scalable. There are no immediate plans to make the storage backend configurable.

A first-party `docker-compose.yml` file is provided to run Heartbeat alongside Postgres on the same docker network on
the same host. Any other setup requires that you bring your own database. Please consult the PostgreSQL documentation
for the same.

[releases]: https://github.com/lmaotrigine/heartbeat/releases
[PostgreSQL]: https://www.postgresql.org
[feature flags]: https://doc.rust-lang.org/stable/cargo/reference/features.html
[shields.io]: https://shields.io
[build arguments]: https://docs.docker.com/build/guide/build-args/
[license]: https://github.com/lmaotrigine/blob/main/LICENSE
