#!/bin/bash
set -e

source scripts/virt_common.sh

# Source: Setup adapted from https://wiki.cfdata.org/pages/viewpage.action?pageId=1187491234

# Avg Cloudflare latency is ~100ms
LATENCY="50ms"
LOSSMODEL="random 0%"
# set limit to "infinite"
LIMIT="1000000"

# ip netns exec m2_ns tc qdisc del dev veth_m2_m3 root
# ip netns exec m2_ns tc qdisc del dev veth_m2_m1 root

modprobe sch_netem

ip netns exec $NS_M2 tc qdisc add dev $VETH_M2_M3 root handle 1: netem limit $LIMIT delay $LATENCY loss $LOSSMODEL
ip netns exec $NS_M2 tc qdisc add dev $VETH_M2_M1 root handle 1: netem limit $LIMIT delay $LATENCY loss $LOSSMODEL

# examples
# LOSSMODEL="random 3%"
# LOSSMODEL=”gemodel 0.1% 1%"
