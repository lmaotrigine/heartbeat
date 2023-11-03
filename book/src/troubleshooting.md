# Troubleshooting

The first step to debugging issues with your installation is to look at the logs. To make it less of a privacy law
hassle, all logs exist ephemerally only in the console I/O. i.e. they are streamed to stdout/stderr depending on their
origin and severity. Fortunately, inspecting them is easy enough. If you are simply running the server in the background
using screen/tmux you can switch to the appropriate windows to see them. If you are using Docker, you can view them
using `docker logs`, or `docker compose logs` if you are using `docker-compose`. If you are using a process manager like
systemd or pm2, you can check your journal, or whatever logging facility it provides. The pertinent error messages will
most likely be helpful enough to find out and fix your problem.

By default, the maximum log level is `INFO` for release builds and `DEBBUG` for debug builds. You should be using
release builds in production. To control the log level, set the `RUST_LOG` environment variable to an acceptable value.
Consult [this documentation][tracing] for more information.

Some common pitfalls are documented here, with suggested debugging solutions. If your issue is not listed here and you
cannot figure it out, we're happy to help. Open an issue or discussion on GitHub, provide us with the relevant logs and
reproduction steps and we will do our best to help you out

## Frequent Problems

### Can't connect using Docker

The default port is `6060`. You should map this port to a host port (preferably the same one). If you still cannot
access it from `localhost`, check `docker logs` or `docker compose logs` to verify that the server is listening where
you expect it to, and that the configuration is laoded correctly.

### Can't connect while not using Docker

Without Docker's networking shenanigans, you should be able to access the server on `localhost` always. Ensure that the
server hasn't prematurely exited due to an error or signal. If not, please check that the bind address is the same one
that you are trying to access.

### 401 errors on /api/beat

When you believe you have the right token but the server is rejecting it, it is likely that the token was regenerated at
some point. Regenerate it once more to ensure that you note it down this time.

### 401 errors on /api/device routes

If the secret key isn't being recognized as valid by the server, try temporarily overriding it using environment
variables or command line flags, and see if the issue persists. This will likely help identify the cause of the issue if
it isn't just a typo.

### Config values from TOML file not being read on Docker

Make sure your `config.toml` file is mounted at `/usr/local/share/heartbeat/config.toml`. This is where the executable
looks for the config file by default. If you overrode this, make sure the mount location matches with the override.

### `error communicating with database: failed to lookup address information: Name or service not known`

Heartbeat could not establish a connection to the PostgreSQL server. Make sure that the server is online and reachable
from the network Heartbeat is running on. If there is another message indicating secure connection failure, ensure that
the `sqlx-tls` feature is enabled in your binary. In release archives, this is always the case, but in Docker, it
requires passing it to the `FEATURES` build argument.

### `pool timed out while waiting for an open connection`

Your PostgreSQL server is possibly at capacilty for maximum connections. Check the server logs or try again after some
time. Failing that, enable debug logging

[tracing]:
    https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html#filtering-events-with-environment-variables
