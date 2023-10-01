-- Copyright (c) 2023 Isis <root@5ht2.me>
--
-- This Source Code Form is subject to the terms of the Mozilla Public
-- License, v. 2.0. If a copy of the MPL was not distributed with this
-- file, You can obtain one at http://mozilla.org/MPL/2.0/.

CREATE SCHEMA IF NOT EXISTS heartbeat;

BEGIN;
ALTER TABLE devices SET SCHEMA heartbeat;
ALTER TABLE beats SET SCHEMA heartbeat;
ALTER TABLE stats SET SCHEMA heartbeat;
END;
