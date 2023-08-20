# API Spec and Client Information

## API Reference

This service exposes an API that clients for all platforms are expected to comply with.

- `POST` `/api/beat`: Ping the server indicating "online" status at this instant.

  **Headers**

    - `Authorization`: A device token (required)

  **Responses**

    - `200`: A Unix timestamp of the time the beat was registered
    - `401`: Invalid or no token provided

- `POST` `/api/devices`: Registers a new device

  **Headers**

    - `Authorization`: The configured secret key for the server.

  **Body**: `application.json`

  A JSON string with a `name` field containing the common name of the device.

  **Responses**

    - `200`: A JSON string containing the device ID, name and token. The token will not be shown again and isn't exposed
      by any endpoint.
    - `401`: Invalid or no secret key provided.
    - `400`: Invalid JSON body

- `GET` `/api/stats`: Retrieves various statistics about the server.

  **Responses**

    - `200`: A JSON string containing the statistics. This schema is to be considered unstable and prone to breakage
      until a stable version is released. You may refer to the source to get the current schema.

## Existing clients

### Android

A [`Tasker`](https://tasker.joaoapps.com/) package can be downloaded from
[TaskerNet][taskernet-link]. There is a task which launches a scene that allows
customizing the server URL and token for your device. You may want to add a shortcut
widget of that task to your home screen.

[taskernet-link]:
    https://taskernet.com/shares/?user=AS35m8lYWmKlKnpucO4NKAF5nrvpAAJ9k0B16Xq4oGo55MJi%2Fne5EtkyyRTuOR565VRqEmzf468J&id=Project%3AHeartbeat
