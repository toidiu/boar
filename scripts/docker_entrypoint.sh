#!/bin/sh
# Container entrypoint for both boar services.
#
# - Chowns /boar/report back to the host user (HOST_UID:HOST_GID, default
#   1000:1000) on shell exit, so bind-mounted output isn't root-owned on the
#   host. No-op on boar-server, where nothing writes to /boar/report.
# - Optionally sleeps before exec'ing the payload, to give a peer service a
#   grace period to come up. Configure with BOAR_STARTUP_WAIT (seconds).

set -e

chown_reports() {
    chown -R "${HOST_UID:-1000}:${HOST_GID:-1000}" /boar/report 2>/dev/null || true
}
trap chown_reports EXIT

if [ "${BOAR_STARTUP_WAIT:-0}" -gt 0 ] 2>/dev/null; then
    sleep "${BOAR_STARTUP_WAIT}"
fi

"$@"
