#!/bin/bash
# Tear down whatever docker_tc.sh installed. Never fails — boar calls this
# before setup to clear stale state, so missing qdiscs aren't an error.
set +e

DEV="${BOAR_DEV:-eth0}"
IFB="${BOAR_IFB:-ifb0}"

tc qdisc del dev $DEV ingress 2>/dev/null
tc qdisc del dev $DEV root 2>/dev/null
tc qdisc del dev $IFB root 2>/dev/null
ip link set dev $IFB down 2>/dev/null

exit 0
