# [heartbeat](https://hb.5ht2.me)

<!-- badges -->
[![Last Online](https://hb.5ht2.me/badge/last-seen?bypass-cache)](https://hb.5ht2.me)
[![Total Beats](https://hb.5ht2.me/badge/total-beats?bypass-cache)](https://hb.5ht2.me)
[![Docker Build](https://img.shields.io/github/actions/workflow/status/lmaotrigine/heartbeat/docker.yml?branch=main&logo=docker&logoColor=white)](https://github.com/lmaotrigine/heartbeat/actions/workflows/docker.yml)
[![Lint](https://img.shields.io/github/actions/workflow/status/lmaotrigine/heartbeat/lint.yml?branch=main&label=lint&logo=github&logoColor=white)](https://github.com/lmaotrigine/heartbeat/actions/workflows/lint.yml)
<!-- end badges -->

A service inspired by [5HT2B/heartbeat](https://github.com/5ht2b/heartbeat) implemented in Rust.

## Improvements

This implements a few additional features when compared to upstream. Some of these are still open issues, and may be
implemented soon.

- Token-per-device
- Persistence improvements with a more robust database
- Built-in support for SVG badge generation (using dynamic badges from shields.io leads to long and clunky URLs).
- Hierarchical configuration.

## Compatibility

This service makes some opinionated design changes from upstream in relation to stats, and how devices are added, and
renames the `Auth` header to the more widely accepted `Authorization`, hence existing clients need a bit of tweaking to
be compatible with this server.

## Usage

See [`docs/usage.md`](docs/usage.md) for compilation and usage instructions, and [`docs/client.md`](docs/client.md) for
API reference and client information.
