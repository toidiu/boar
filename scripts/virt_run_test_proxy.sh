#!/bin/bash
set -e

source scripts/virt_common.sh

# ========= args
PORT="${1:-$PORT_S1}"
NS_NAME="${2:-$NS_S1}"

NS="ip netns exec $NS_NAME"
# ========= args

# Build binary
cargo build --package test-proxy

# Examples:
#
# flamegraph
# sudo $NS .cargo/bin/flamegraph --  ./target/release/test-proxy -c bbr2_gcongestion -a 0.0.0.0:$PORT

sudo QLOGDIR=qlog $NS ./target/debug/test-proxy -c bbr2_gcongestion -a 0.0.0.0:$PORT
