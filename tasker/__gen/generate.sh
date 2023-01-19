#!/bin/bash

HOST="{HB_HOSTNAME:-'hb.5ht2.me'}"
AUTH_TOKEN="${HB_TOKEN}"

_tasker="$(dirname "$(dirname "$0")")"
sed "s#{HOSTNAME}#${HOST}#; s#{AUTH_TOKEN}#${AUTH_TOKEN}#" "${_tasker}/__gen/Heartbeat.prf.xml" \
    > "${_tasker}/Heartbeat.prf.xml"
