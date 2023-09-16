-- This file is part of https://github.com/lmaotrigine/heartbeat.
-- Copyright (c) 2023 Isis <root@5ht2.me>
--
-- This Source Code Form is subject to the terms of the Mozilla Public
-- License, v. 2.0. If a copy of the MPL was not distributed with this
-- file, You can obtain one at https://mozilla.org/MPL/2.0/.

CREATE SCHEMA heartbeat;

CREATE TABLE heartbeat.devices (
  id BIGINT PRIMARY KEY,
  name TEXT,
  token TEXT NOT NULL,
  num_beats BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX devices_token_idx ON heartbeat.devices (token);

CREATE TABLE heartbeat.beats (
  device BIGINT NOT NULL REFERENCES heartbeat.devices(id) ON DELETE CASCADE,
  time_stamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() at time zone 'utc'),
  PRIMARY KEY (device, time_stamp)
);

CREATE TABLE heartbeat.stats (
  _id SMALLINT PRIMARY KEY,
  total_visits BIGINT NOT NULL DEFAULT 0,
  server_start_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
  longest_absence INTERVAL NOT NULL DEFAULT '0 seconds'
);
