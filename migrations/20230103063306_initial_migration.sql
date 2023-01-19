-- Add migration script here
CREATE TABLE devices (
  id BIGINT PRIMARY KEY,
  name TEXT,
  token TEXT NOT NULL
);

CREATE INDEX devices_token_idx ON devices (token);

CREATE TABLE beats (
  device BIGINT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
  time_stamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() at time zone 'utc'),
  PRIMARY KEY (device, time_stamp)
);

-- this table seems a bit sparse, but since we don't want to log IPs etc
-- there isn't much else to persist as far as I can tell.
CREATE TABLE stats (
  _id SMALLINT PRIMARY KEY,
  total_visits BIGINT NOT NULL DEFAULT 0,
  server_start_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (NOW() AT TIME ZONE 'utc')
);
