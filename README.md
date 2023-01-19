# [heartbeat](https://hb.5ht2.me)

<!-- badges -->
[![Last Online](https://hb.5ht2.me/badge/last-seen?bypass-cache)](https://hb.5ht2.me)
[![Total Beats](https://hb.5ht2.me/badge/total-beats?bypass-cache)](https://hb.5ht2.me)
[![Docker Build](https://img.shields.io/github/actions/workflow/status/lmaotrigine/heartbeat/docker.yml?branch=main&logo=docker&logoColor=white)](https://github.com/lmaotrigine/heartbeat/actions/workflows/docker.yml)
[![Lint](https://img.shields.io/github/actions/workflow/status/lmaotrigine/heartbeat/lint.yml?branch=main&label=lint&logo=github&logoColor=white)](https://github.com/lmaotrigine/heartbeat/actions/workflows/lint.yml)
<!-- end badges -->

A service inspired by [5HT2B/heartbeat](https://github.com/5ht2b/heartbeat) implemented in Rust.

## Compatibility

This service makes some opinionated design changes from upstream in relation to stats, and how devices are added.
However, the `/api/beat` endpoint functions the same, so clients that handle pinging this endpoint will be compatible.


### TODO

- [ ] Documentation
- [x] Webhook support
- [ ] Tests
