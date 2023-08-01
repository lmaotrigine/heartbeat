-- Add migration script here
CREATE TABLE IF NOT EXISTS devices (
  id BIGINT PRIMARY KEY,
  name TEXT,
  token TEXT NOT NULL,
  num_beats BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS devices_token_idx ON devices (token);

CREATE TABLE IF NOT EXISTS beats (
  device BIGINT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
  time_stamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() at time zone 'utc'),
  PRIMARY KEY (device, time_stamp)
);

CREATE TABLE IF NOT EXISTS stats (
  _id SMALLINT PRIMARY KEY,
  total_visits BIGINT NOT NULL DEFAULT 0,
  server_start_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc'),
  longest_absence INTERVAL NOT NULL DEFAULT '0 seconds'
);