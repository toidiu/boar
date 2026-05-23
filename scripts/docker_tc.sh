#!/bin/bash
set -e

# Apply tc shaping inside the boar-client container. The container's eth0 is
# the veth into the docker user-defined bridge that links it to the server
# container, so any shaping here lands on the server<->client path.
#
# Ingress (downloads from the server) is shaped by redirecting eth0 ingress
# into ifb0 and applying htb (rate) + netem (delay/loss) there. Egress gets
# fq to match the pacing-friendly setup the old ns_s1 used.
#
# Defaults mirror the legacy virt_config_tc.sh args: latency, rate, loss.

LATENCY="${1:-50ms}"
RATE="${2:-20mbit}"
LOSSMODEL="${3:-random 0%}"
DEV="${BOAR_DEV:-eth0}"
IFB="${BOAR_IFB:-ifb0}"

modprobe ifb numifbs=1 2>/dev/null || true
modprobe sch_netem 2>/dev/null || true

ip link show $IFB >/dev/null 2>&1 || ip link add $IFB type ifb
ip link set dev $IFB up

# Wipe any prior state so this script is idempotent across boar runs.
tc qdisc del dev $DEV ingress 2>/dev/null || true
tc qdisc del dev $DEV root 2>/dev/null || true
tc qdisc del dev $IFB root 2>/dev/null || true

# Redirect $DEV ingress into $IFB so we can attach an egress qdisc to it.
tc qdisc add dev $DEV handle ffff: ingress
tc filter add dev $DEV parent ffff: protocol ip u32 match u32 0 0 \
    action mirred egress redirect dev $IFB

# Shape the redirected ingress: htb caps throughput, netem injects RTT/loss.
tc qdisc add dev $IFB root handle 1: htb default 99
tc class add dev $IFB parent 1: classid 1:99 htb quantum 1514 rate $RATE ceil $RATE
tc qdisc add dev $IFB parent 1:99 handle 99: netem limit 1000000 delay $LATENCY loss $LOSSMODEL

# Egress on the real device: fq for pacing.
tc qdisc add dev $DEV root fq

echo "[docker_tc] $DEV ingress -> $IFB (rate=$RATE delay=$LATENCY loss='$LOSSMODEL'), $DEV egress=fq"
