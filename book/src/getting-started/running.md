# Running

Heartbeat requires some configuration information at runtime to communicate with the database, set up routing, and
finally to listen and serve. Sensible defaults for the various options are picked if not provided. Detailed information
on configuring the server are provided in a [separate section][configuring].

A sample configuration file is distributed with the source, and in release archives. You can view the latest version
online on [GitHub](https://github.com/lmaotrigine/heartbeat/blob/main/config.example.toml). This file has comments
explaining the various fields.

After configuring the server, it can be simply started in the background.

The quickest way to get started by using Docker is to clone the repository and run

```console
$ docker compose up
```

And visit http://127.0.0.1:6060 in a browser to check if everything went well.

## Registering your first device

Assuming that you set a value for the `secret_key` parameter – there are several ways to generate one, one of which is to
run `heartbeat gen-key`, which is distributed in release archives and can also be built from source – you can now hit
the `/api/devices` endpoint to register your first device.

First, you'll need a name for your device, let's call it `Laptop` (but you can get creative!)

Using the command line, with `curl` installed (it should be by default on Windows and any sane Linux distro, and on
macOS will prompt you to install Xcode command line tools), you can just run

```console
$ curl -XPOST -H 'Authorization: <my_secret_key>' -H 'Content-Type: application/json' -d '{"name": "Laptop"}' http://127.0.0.1:6060/api/devices
```

That's a long one! You will probably want to make a convenience wrapper for this. We don't provide one out of the box
because various users might have multiple very creative ways to store their secrets, and we leave it to them to tailor
it to their needs.

Once you've registered a device, you will get a response back with a `token` field in it. Note this down because you
cannot retrieve it from the server at any other time. You can then use this token as the `Authorization` header when you
send your first beat.

Use this token wherever appropriate when you set up the [client](../clients/index.md) for your device, and test it out
by sending your first beat. If all goes well, the homepage should refresh with new statistics.

[configuring]: ../configuration.md
