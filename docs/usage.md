# Usage

## Obtaining the source

Ensure [`git`](https://git-scm.com) is installed and in your path.

```sh
# you can add --depth 1 to this command if you don't care about the commit history
git clone https://github.com/lmaotrigine/heartbeat
cd heartbeat
```

## Setting up

Refer to the [server setup documentation](./server.md) for instructions to get the server up and running.

## Testing

To test locally, you will need to first add a device.

```sh
# the key set in the secret_key field of your config file
curl -X POST -H "Authorization: $your_secret_key" -H "Content-Type: application/json" -d '{"name": "Laptop"}' localhost:6060/api/devices
```

You should receive a JSON response with the details of your newly registered device. Note the `token` field as this will
not be shown again.

To test a ping locally:

```sh
curl -X POST -H "Authorization: $your_device_token" localhost:6060
```

You should get a response with the Unix timestamp of your ping.

You can open `localhost:6060` in a browser to see the webpage.

## Debugging

- Can't connect using Docker

  The default port is `6060`. You should be able to access `localhost:6060`.
  If you are unable to connect from localhost, check `docker-compose logs` for issues.

- Can't connect when running the binary directly

  If running the binary is not throwing any errors, make sure you are connecting to the correct address and port as
  configured in Rocket.toml.

- Can't read the config file in Docker

  The compose file attaches a file called `config.toml` in the current directory as a volume. Make sure this file exists
  and contains the configuration you want. If you are not using the compose file, make sure to mount your config file to
  `/usr/local/share/heartbeat/config.toml`.

- `failed to initialize database: failed to initialize database: pool timed out while waiting for an open connection`

  The service can't connect to the PostgreSQL server. If you're not using Docker, make sure that your PostgreSQL server
  is running and listening at the configured address before running heartbeat.

## Feature flags

Some functionality is gated behind feature flags, with some enabled by default. You may toggle these using command line
options at build time. The binary built for Docker enables the default feature set.

[Cargo documentation](https://doc.rust-lang.org/cargo/reference/features.html)

- `badges` (*default*): Enables the `/badge/last-seen` and `/badge/total-beats` routes which generate embeddable SVG badges similar
  to shields.io.

- `webhook` (*default*): Enables logging certain events to a Discord webhook. See the config file comments for more info.

- `sqlx-tls`: Enables the `tls-rustls` feature of `sqlx`, which allows you to connect to PostgreSQL servers over
  SSL/TLS.

- `migrate`: Enables the `migrate` feature of `sqlx`. This is required to build/run the embedded migrations binary,
  which is useful if you don't (want to) have `sqlx-cli` installed globally.
