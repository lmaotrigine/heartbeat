# API Reference

This document is a hand-crafted reference of the entire public API exposed by the server. Care must be taken to keep
this document up-to-date whenever changes are made to the request or response types, or new routes are added or existing
routes removed.

## Devices

Actions relating to devices. These routes are non-existent if the `secret_key` configuration parameter is left empty.

### `POST /api/devices`

Register a new device.

- Authentication: `Authorization` header with the same value as the `secret_key` configuration parameter of the server.
- Request body:
  - Content Type: `application/json`
  - Schema: `{name: string}`
  - Example: `{"name": "Laptop"}`
- Response:
  - Content Type: `application/json`
  - Schema:
    ```ts
    {
      id: number,
      name: string,
      token: string
    }
    ```
  - Example:
    ```json
    {
      "id": 0,
      "name": "Laptop",
      "token": "barfooed"
    }
    ```
- Errors
  - `400`: Invalid request body
  - `401`: Invalid or missing Authorization header
  - `405`: Not a POST request

### `POST /api/devices/:id/token/generate`

(Re)generate the token for a registered device.

- Authentication: `Authorization` header with the same value as the `secret_key` configuration parameter of the server.
- Path parameters:
  - `id`: The ID of the device to regenerate the token for
- Response:
  - Content Type: `application/json`
  - Schema:
    ```ts
    {
      id: number,
      name: string,
      token: string
    }
    ```
  - Example:
    ```json
    {
      "id": 0,
      "name": "Laptop",
      "token": "barfooed"
    }
    ```
- Errors:
  - `401`: Invalid or missing Authorization header
  - `404`: Device with the provided ID does not exist
  - `405`: Not a POST request

## Beats

Actions that a [client](./index.md) will have to implement.

### `POST /api/beat`

- Authentication: `Authorization` header with the device token which was obtained during registration. If regenerated,
  old values of the token are no longer considered valid.
- Response:
  - Content Type: `text/plain`
  - Schema: A Unix timestamp corresponding to the time the beat was acknowledged.
  - Example: `1698915036`
- Errors:
  - `401`: Invalid or missing Authorization header
  - `405`: Not a POST request

## Statistics

Operations to retrieve statistics about the server.

### `GET /api/stats`

- Authentication: none
- Response:
  - Content Type: `application/json`
  - Schema:
    ```ts
    {
      last_seen: number, // Unix timestamp of last beat
      last_seen_relative: number, // number of whole seconds since last beat
      longest_absence: number, // duration of longest absence ever (in seconds)
      num_visits: number, // number of visits to the site (not including API calls)
      total_beats: number, // number of beats since the server started operating
      devices: Device[], // array of devices, see type definition below.
      uptime: number, // number of whole seconds since the server was first set up
    }
    /////
    type Device = {
      id: number,
      name: string,
      num_beats: number,  // number of beats by this device since the server started operating
    }
    ```
  - Example:
    ```json
    {
      "last_seen": 1698915320,
      "last_seen_relative": 41,
      "longest_absence": 84898,
      "num_visits": 1628,
      "total_beats": 120062,
      "devices": [
        {
          "id": 0,
          "name": "Laptop",
          "last_beat": 1698825626,
          "num_beats": 36308
        },
        {
          "id": 1,
          "name": "Phone",
          "last_beat": 1698825626,
          "num_beats": 639
        },
        {
          "id": 2,
          "name": "Workstation",
          "last_beat": 1698915320,
          "num_beats": 83115
        }
      ],
      "uptime": 8082297
    }
    ```

### `GET /api/stats/ws`

A WebSocket endpoint to stream statistics. Responses are JSON strings in the same schema as above, and are streamed at
the rate of 1/second.
