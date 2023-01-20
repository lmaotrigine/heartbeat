#!/bin/bash

# Copyright (c) 2023 VJ <root@5ht2.me>
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

HOST="{HB_HOSTNAME:-'hb.5ht2.me'}"
AUTH_TOKEN="${HB_TOKEN}"

_tasker="$(dirname "$(dirname "$0")")"
sed "s#{HOSTNAME}#${HOST}#; s#{AUTH_TOKEN}#${AUTH_TOKEN}#" "${_tasker}/__gen/Heartbeat.prf.xml" \
    > "${_tasker}/Heartbeat.prf.xml"
