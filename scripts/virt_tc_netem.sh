#!/bin/bash
set -e

source scripts/virt_common.sh


#######################
# ARGS
############
LATENCY="${1:-50ms}"
LOSSMODEL="${2:-random 0%}"
# set limit to "infinite"
LIMIT="1000000"

# DEBUG
# echo "virt_config_tc. latency: $LATENCY. all args: $@"
############
# ARGS END
#######################


# ip netns exec m2_ns tc qdisc del dev veth_m2_m3 root
# ip netns exec m2_ns tc qdisc del dev veth_m2_m1 root

modprobe sch_netem

ip netns exec $NS_M2 tc qdisc add dev $VETH_M2_M3 root handle 1: netem limit $LIMIT delay $LATENCY loss $LOSSMODEL
ip netns exec $NS_M2 tc qdisc add dev $VETH_M2_M1 root handle 1: netem limit $LIMIT delay $LATENCY loss $LOSSMODEL

# examples
# LOSSMODEL="random 3%"
# LOSSMODEL=”gemodel 0.1% 1%"
