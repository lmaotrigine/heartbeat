-- This Source Code Form is subject to the terms of the Mozilla Public
-- License, v. 2.0. If a copy of the MPL was not distributed with this
-- file, You can obtain one at http://mozilla.org/MPL/2.0/.

ALTER TABLE devices ADD COLUMN num_beats BIGINT NOT NULL DEFAULT 0;
WITH nb AS (
  SELECT device, count(*) AS num_beats FROM beats GROUP BY device
)
UPDATE devices SET num_beats = nb.num_beats FROM nb WHERE devices.id = nb.device;
ALTER TABLE stats ADD COLUMN longest_absence INTERVAL NOT NULL DEFAULT '0 seconds';
