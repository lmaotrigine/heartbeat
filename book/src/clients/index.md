# Clients

First-party clients are available for a variety of platforms. The future of these implementations and the possibility of
new ones being added depend entirely on the bandwidth of the maintainers and the possibility of testing them, including
access to relevant hardware and software. Currently, supported client platforms are:

- Linux: via [`heartbeat-unix`][unix], only supports `X.org`. Support for Wayland is planned, but not immediately in the
  works, and help in this area would be appreciated.
- macOS: via [`heartbeat-unix`][unix].
- Android: A [Tasker] project bundle is available on [TaskerNet]. An Android app is currently under development, but
  there is no ETA.
- Windows: via [`heartbeat-windows`][windows], tested on the latest stable build of Windows 10 and latest insiders build of
  Windows 11.

[unix]: https://github.com/lmaotrigine/heartbeat-unix
[windows]: https://github.com/5HT2B/heartbeat-windows
[Tasker]: https://tasker.joaoapps.com/
[TaskerNet]: https://taskernet.com/shares/?user=AS35m8lYWmKlKnpucO4NKAF5nrvpAAJ9k0B16Xq4oGo55MJi%2Fne5EtkyyRTuOR565VRqEmzf468J&id=Project%3AHeartbeat

Implementing your own client to support other platforms is a straightforward process. You must implement the
[API](./api.md), specifically for the `/api/beat` endpoint, and hit it every so often while the device is actively being
used. This can be determined by various factors such as the last time an input device was used, last time the screen was
unlocked, the last time the device was awakened from an idle state, etc. At the very least you will need to make network
requests, so devices without this capability cannot be supported.
